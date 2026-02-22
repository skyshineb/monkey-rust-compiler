let outer = fn(a) { fn(b) { fn(c) { a + b + c } } };
outer(1)(2)(3);
