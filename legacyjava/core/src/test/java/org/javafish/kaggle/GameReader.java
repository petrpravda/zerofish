package org.javafish.kaggle;

import org.tukaani.xz.XZInputStream;

import java.io.BufferedReader;
import java.io.FileInputStream;
import java.io.IOException;
import java.io.InputStreamReader;
import java.security.SecureRandom;
import java.util.Iterator;
import java.util.Optional;

public class GameReader implements AutoCloseable, Iterable<Optional<PsychoLine.ParsedLine>> {
    private final InputStreamReader inputStreamReader;
    private final FileInputStream fileInputStream;
    private final XZInputStream xzInputStream;
    private final BufferedReader bufferedReader;
    private final PsychoLine psychoLine = new PsychoLine();

    public GameReader() {
        //PsychoLine psychoLine = new PsychoLine();
        try {
            fileInputStream = new FileInputStream("../puzzle-mania-analysis/all_with_filtered_anotations_since1998.txt.xz");
            xzInputStream = new XZInputStream(fileInputStream);
            inputStreamReader = new InputStreamReader(xzInputStream);

            bufferedReader = new BufferedReader(inputStreamReader);
            for (int i = 0; i++ < 5; ) {
                bufferedReader.readLine();
            }
        } catch (IOException e) {
            throw new IllegalStateException(String.format("Cannot create GameReader: %s", e.getMessage()));
        }

    }

    @Override
    public void close() {
        try {
            bufferedReader.close();
            inputStreamReader.close();
            xzInputStream.close();
            fileInputStream.close();
        } catch (IOException e) {
            throw new IllegalStateException(String.format("Cannot auto close GameReader: %s", e.getMessage()));
        }
    }

    public Optional<PsychoLine.ParsedLine> readGame() {
        try {
            String line = bufferedReader.readLine();
            if (line == null) {
                return null;
            }
            Optional<PsychoLine.ParsedLine> game = psychoLine.parse(line);
            return game;
        } catch (IOException e) {
            throw new IllegalStateException(String.format("Cannot read game: %s", e.getMessage()));
        }
    }

    @Override
    public Iterator<Optional<PsychoLine.ParsedLine>> iterator() {
        return new Iterator<Optional<PsychoLine.ParsedLine>>() {
            private Optional<PsychoLine.ParsedLine> game;

            @Override
            public boolean hasNext() {
                game = readGame();
                return game != null;
            }

            @Override
            public Optional<PsychoLine.ParsedLine> next() {
                return game;
            }
        };
    }
}
