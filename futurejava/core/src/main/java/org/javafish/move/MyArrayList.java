package org.javafish.move;

import java.util.Arrays;
import java.util.Iterator;
import java.util.NoSuchElementException;
import java.util.function.Consumer;
import java.util.stream.Stream;

public class MyArrayList<T> implements Iterable<T> {
    private Object[] array;
    private int size = 0;

    public MyArrayList(int initialSize) {
        array = new Object[initialSize];
    }

    public final void add(T move) {
        if (size == array.length) {
            // Resize the array if it's full
            Object[] newArray = new Object[array.length * 2];
            System.arraycopy(array, 0, newArray, 0, array.length);
            array = newArray;
        }
        array[size] = move;
        size++;
    }

    public final int size() {
        return size;
    }

    @SuppressWarnings("unchecked")
    public final T get(int i) {
        if (i < 0 || i >= size) {
            throw new IndexOutOfBoundsException();
        }
        return (T) array[i];
    }

    @SuppressWarnings("unchecked")
    public final Stream<T> stream() {
        return (Stream<T>) Arrays.stream(array, 0, size);
    }

    @Override
    public final Iterator<T> iterator() {
        return new Iterator<T>() {
            private int index = 0;

            @Override
            public boolean hasNext() {
                return index < size;
            }

            @SuppressWarnings("unchecked")
            @Override
            public T next() {
                if (!hasNext()) {
                    throw new NoSuchElementException();
                }
                return (T) array[index++];
            }
        };
    }

    @SuppressWarnings("unchecked")
    public final void forEach(Consumer<? super T> action) {
        for (int i = 0; i < size; i++) {
            action.accept((T) array[i]);
        }
    }
}

