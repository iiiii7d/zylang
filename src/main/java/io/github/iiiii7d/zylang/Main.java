package io.github.iiiii7d.zylang;

import net.sourceforge.argparse4j.ArgumentParsers;
import net.sourceforge.argparse4j.inf.*;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;

// for my reference
//String content = Files.readString(Path.of("test.zy"), StandardCharsets.US_ASCII);

public class Main {
    public static final String version = "0.0.0";

    public static ArrayList<Character> compile(String in) {
        return Lexer.lex(in);
    }

    public static void main(String[] args) {
        ArgumentParser parser = ArgumentParsers.newFor("zy").build()
                .description("Command line tool for Zy");
        Subparsers subparsers = parser.addSubparsers().help("sub-command help").dest("cmd");

        subparsers.addParser("version").help("Displays version info");

        Subparser runParser = subparsers.addParser("run").help("Run a .zy file");
        runParser.addArgument("file").type(String.class).help("The .zy to run");

        Subparser compileParser = subparsers.addParser("compile").help("Compile a .zy file into a .zyi file");
        compileParser.addArgument("file").type(String.class).help("The .zy file to compile");

        Subparser interpretParser = subparsers.addParser("interpret").help("Interprets a .zyi file");
        interpretParser.addArgument("file").type(String.class).help("The .zyi file to interpet");

        try {
            Namespace parserArgs = parser.parseArgs(args);

            try {
                switch ((String) parserArgs.get("cmd")) {
                    case "version":
                        System.out.println("Zy version " + Main.version);
                        break;
                    case "run":
                        System.out.println(compile(Files.readString(Path.of((String) parserArgs.get("file")), StandardCharsets.UTF_8)));
                        break;
                    case "compile":
                    case "interpret":
                        System.out.println("Coming soon");
                        break;
                    default:
                        break;
                }
            } catch (IOException e) {
                System.out.println(Ansi.red("Invalid file " + parserArgs.get("file")));
                System.exit(1);
            }
        } catch (ArgumentParserException e) {
            parser.handleError(e);
            System.exit(1);
        }
    }
}
