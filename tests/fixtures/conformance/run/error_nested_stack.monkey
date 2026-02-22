let bad = fn(x) { x + true };
let mid = fn(y) { bad(y) };
mid(1);
