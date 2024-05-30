# TODOs
- [ ] add support for control characters ('\n', '\t', '\r', ...)
- [ ] check all edge cases for correct functionality
- [ ] test performance
- [ ] see if you can somehow improve performance by using clear commands instead of clearing by printing a lot of space characters

## Considerations
- maybe change the system to do save first_change and last_change per line like ncurses does (ldat->firstchar, ldat->lastchar)
