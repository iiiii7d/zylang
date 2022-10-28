use std::ops::Range;

use itertools::Either;

use crate::{
    types::{
        position::GetPosRaw,
        token::{Token, TokenType},
    },
    Element, ZError, ZResult,
};

#[derive(Clone, Debug)]
pub struct Buffer {
    pub content: Vec<Either<Element, Token>>,
    pub cursor: usize,
    started: bool,
    raw: Option<String>,
}
impl Buffer {
    pub fn new(input: Vec<Token>) -> Self {
        Self {
            content: input
                .into_iter()
                .map(Either::Right)
                .collect::<Vec<Either<Element, _>>>(),
            cursor: 0,
            started: false,
            raw: None,
        }
    }
    pub fn next(&mut self) -> Option<Either<Element, Token>> {
        if self.started {
            self.cursor += 1;
        } else {
            self.started = true;
        }
        let next = self.content.get(self.cursor).cloned();
        if let Some(raw) = &mut self.raw {
            if let Some(next) = &next {
                raw.push_str(&match next {
                    Either::Left(c) => c.pos_raw.raw.to_owned(),
                    Either::Right(c) => c.get_raw().into(),
                });
            }
        }
        next
    }
    pub fn next_or_err(&mut self) -> ZResult<Either<Element, Token>> {
        if let Some(c) = self.next() {
            Ok(c)
        } else {
            let curr_pos_raw = match &self.content.last().unwrap() {
                Either::Left(c) => c.pos_raw(),
                Either::Right(c) => c.pos_raw(),
            };
            Err(ZError::error_2_1_0(&curr_pos_raw.raw).with_pos_raw(&curr_pos_raw))
        }
    }
    pub fn prev(&mut self) -> Option<&Either<Element, Token>> {
        if !self.started {
            None
        } else {
            self.content.get(self.cursor - 1)
        }
    }
    pub fn rest_incl_curr(&mut self) -> BufferWindow {
        self.window(self.cursor..self.content.len())
    }
    pub fn next_cursor_pos(&self) -> usize {
        if self.started {
            self.cursor + 1
        } else {
            0
        }
    }
    pub fn reset_cursor(&mut self) {
        self.started = false;
        self.cursor = 0;
    }
    pub fn peek(&self) -> Option<&Either<Element, Token>> {
        self.content.get(self.next_cursor_pos())
    }
    pub fn start_raw_collection(&mut self) {
        self.raw.get_or_insert_with(|| {
            if self.started {
                self.content
                    .get(self.cursor)
                    .map(|c| match c {
                        Either::Left(c) => c.pos_raw.raw.to_owned(),
                        Either::Right(c) => c.get_raw().into(),
                    })
                    .unwrap_or_else(|| "".into())
            } else {
                "".into()
            }
            .to_string()
        });
    }
    pub fn end_raw_collection(&mut self) -> String {
        self.raw.take().unwrap_or_else(|| "".into())
    }
    pub fn window(&self, range: Range<usize>) -> BufferWindow {
        BufferWindow {
            slice: self.content[range.to_owned()].to_owned(),
            range,
        }
    }
    pub fn get_between(
        &mut self,
        start_token: TokenType,
        end_token: TokenType,
    ) -> ZResult<BufferWindow> {
        let mut nest_level = 1usize;
        let start = self.cursor;
        while let Some(ele) = self.next() {
            if let Either::Right(ele) = ele {
                if start_token == end_token {
                    nest_level = usize::from(nest_level != 1);
                } else if ele.ty == Some(start_token) {
                    nest_level += 1
                } else if ele.ty == Some(end_token) {
                    nest_level -= 1
                }
            }
            if nest_level == 0 {
                break;
            }
        }
        if nest_level != 0 {
            todo!("err")
        }
        Ok(BufferWindow {
            slice: self.content[start + 1..self.cursor].to_owned(),
            range: start..self.next_cursor_pos(),
        })
    }
    pub fn get_split(&mut self, divider: TokenType) -> ZResult<BufferWindows> {
        let mut start = self.cursor;
        let mut buffer_windows = vec![];
        while let Some(ele) = self.next() {
            if let Either::Right(ele) = ele {
                if ele.ty == Some(divider) {
                    buffer_windows.push(self.window(start..self.cursor).to_owned());
                    start = self.next_cursor_pos();
                }
            }
        }
        Ok(BufferWindows {
            buffer_windows,
            range: start..self.next_cursor_pos(),
        })
    }
    pub fn get_split_between(
        &mut self,
        start_token: TokenType,
        end_token: TokenType,
        divider: TokenType,
    ) -> ZResult<BufferWindows> {
        let mut nest_level = 1usize;
        let bet_start = self.cursor;
        let mut start = self.cursor + 1;
        let mut buffer_windows = vec![];
        while let Some(ele) = self.next() {
            if let Either::Right(ele) = ele {
                if start_token == end_token {
                    nest_level = usize::from(nest_level != 1);
                } else if ele.ty == Some(start_token) {
                    nest_level += 1
                } else if ele.ty == Some(end_token) {
                    nest_level -= 1
                }
                if nest_level == 1 && ele.ty == Some(divider) {
                    buffer_windows.push(self.window(start..self.cursor).to_owned());
                    start = self.next_cursor_pos();
                }
            }
            if nest_level == 0 {
                break;
            }
        }
        if nest_level != 0 {
            todo!("err")
        }
        Ok(BufferWindows {
            buffer_windows,
            range: bet_start..self.next_cursor_pos(),
        })
    }
    pub fn splice_buffer(&mut self, buffer: BufferWindow) {
        self.content = self.content.to_owned();
        self.cursor = buffer.range.end + buffer.slice.len() - buffer.range.len();
        self.content.splice(buffer.range, buffer.slice);
    }
}

#[derive(Clone)]
pub struct BufferWindow {
    pub slice: Vec<Either<Element, Token>>,
    pub range: Range<usize>,
}
impl BufferWindow {
    pub fn as_buffer(&self) -> Buffer {
        Buffer {
            content: self.slice.to_owned(),
            cursor: 0,
            started: false,
            raw: None,
        }
    }
    pub fn with_as_buffer<T>(&mut self, f: &dyn Fn(&mut Buffer) -> ZResult<T>) -> ZResult<T> {
        let mut buffer = self.as_buffer();
        let res = f(&mut buffer)?;
        let bw = BufferWindow {
            slice: buffer.content,
            range: self.range.to_owned(),
        };
        *self = bw;
        Ok(res)
    }
}

#[derive(Clone)]
pub struct BufferWindows {
    pub buffer_windows: Vec<BufferWindow>,
    pub range: Range<usize>,
}
impl BufferWindows {
    pub fn with_as_buffers<T>(&mut self, f: &dyn Fn(&mut Buffer) -> ZResult<T>) -> ZResult<Vec<T>> {
        self.buffer_windows
            .iter_mut()
            .map(|b| b.with_as_buffer(f))
            .collect::<ZResult<Vec<_>>>()
    }
}
