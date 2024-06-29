package org.javafish.eval.annotation;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.METHOD)
public @interface Parameter {
    String name();
    boolean absolute() default false;
    boolean signed() default false;
    boolean perSide() default false;
}

