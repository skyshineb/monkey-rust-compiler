let f = fn() {
  while (true) {
    if (false) { continue; }
    break;
  }
};
f();
