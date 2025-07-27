# Issues

## Ready to Develop

- [x] Stop using different color in the current line and status bar. Use the same color as the rest of the text in the request/response buffer. Refer to the MVC code for the details.
- [x] Align The mode, pane, and position indicators in the status bar to the right.
- [x] Implement `:q` and `:q!` to quit the application.
- [ ] Support `i` command to insert text at the current cursor position in the request buffer.
- [ ] Support `a` command to insert text next to the current cursor position in the request buffer.
- [ ] Restore the HTTP status code with signal light, message, turn around time in the status bar. Show it before the mode indicator. Refer to the MVC code for the details.
- [ ] Show the logical line number in the request/response buffer. Minimal width for the line number should be 3. The tilda should be placed in the first column. Refer to the MVC code for the details.

## Backlog

- [ ] Rename command terminology for clarity: h/j/k/l as "motions", i/a/A as "editing commands", :q/:w as "ex commands", Ctrl+C as "application commands"
- [ ] Implement `b` to skip to the previous word in the request/response buffer.
- [ ] Implement `0` and `Home` to go to the beginning of the current line in the request/response buffer.
- [ ] Implement `$` and `End` to go to the end of the current line in the request/response buffer.
- [ ] Implement `Shift + a` to append text to the end of the current line in the request buffer.
- [ ] Implement `Shift + i` to insert text at the beginning of the current line in the request buffer.
- [ ] Implement `Ctrl + r` to refresh the current pane.
- [ ] Implement `Ctrl + l` to clear the current pane.
- [ ] Implement `v` to enter visual mode in the request/response buffer.
- [ ] Implement `y` to copy the selected text in the request/response buffer to the clipboard.
- [ ] Implement `yy` to copy the current line in the request/response buffer to the clipboard.
- [ ] Implement `dd` to cut the current line in the request/response buffer to the clipboard.
- [ ] Implement `x` to delete the current character in the request buffer.
- [ ] Implement `p` to paste the text in the clipboard in the request buffer.
- [ ] Implement `Shift + p` to paste the copied line before the current line in the request buffer.
- [ ] Implement `Shift + d` to cut the current character to the end of the line in the request buffer.
- [ ] Implement syntax highlighting for HTTP requests in the request buffer.

## Done

- [x] The last HTTP status code, message, and turn around time must be retained and aligned to the right in the status bar.
- [x] initial width for the line number must be 3. The tilda should be placed in the first column.
- [x] src/repl/command.rs can be renamed as commands.rs and we can then migrate mod declaration from repl.rs to this.
- [x] integration tests must incorporate the check for the screen refresh. This could be done by a mock framework that hooks the invocation of a refresh method and maintain/check its call count.
- [x] SwitchPaneCommand in the movement.rs must be pushed out to window.rs.
- [x] Implement `Ctrl + j` to expand the response pane for one line. It goes up to the request pane shrinks to three lines.
- [x] Implement `Ctrl + k` to shrink the response pane for one line. It goes down to the response pane shrinks to three lines.
- [x] Implement `:r` to show/hide the response pane.
- [x] Fix the flicking issue when the cursor is moved in the request pane. It only happens in the request pane, regardless of the response pane being shown or not.
- [x] Hide the cursor when it is switched to the command mode. Restore the cursor when it is switched back to the normal mode.
- [x] Implement `gg` to go to the top of the current pane.
- [x] Implement `G` to go to the bottom of the current pane.
- [x] Make arrow keys work in the request/response pane, regardless of the mode.
- [x] Implement `w` to skip to the next word in the request/response buffer.
