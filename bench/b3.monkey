let mk = fn(a) {
  fn(b) {
    let arr = [a, b, a + b, a * b];
    {"sum": arr[2], "len": len(arr)}
  }
};
let f = mk(8);
let h = f(13);
h["sum"] + h["len"];
