# TODOs
- [ ] add support for control characters ('\n', '\t', '\r', ...)
- [ ] check all edge cases for correct functionality
- [ ] test performance
- [ ] see if you can somehow improve performance by using clear commands instead of clearing by printing a lot of space characters
- [ ] clear based on the current colors (since terminal emulators seem to do the same)

## Considerations
- maybe change the system to do save first_change and last_change per line like ncurses does (ldat->firstchar, ldat->lastchar)
