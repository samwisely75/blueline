# Issues

## Ready to Develop

## Backlog

- [ ] Remove headers instruction from the request buffer. For example, the following lines should be removed:

  ```bash
  Host: example.com
  Content-Type: application/json
  ```

  from the example of the request written in the `movement.feature` file. This is an extra imaginary feature that the AI came up which I didn't authorize.
  
- [ ] Support `Ctrl + f` to scroll down one page in the request/response pane.
- [ ] Support `Ctrl + b` to scroll up one page in the request/response pane.
- [ ] Support `Ctrl + d` to scroll down half a page in the request/response pane.
- [ ] Support `Ctrl + u` to scroll up half a page in the request/response pane.
- [ ] Show "Executing..." in the status bar when the request is being processed. The response pane should be cleared and another request submission should be ignored until the response is received.
- [ ] Dim the Status Bar when it's not in focus to reduce visual clutter.
- [ ] Support `w` to skip to the next word in the request/response buffer.
- [ ] Support `b` to skip to the previous word in the request/response buffer.
- [ ] Support `e` to skip to the end of the current word in the request/response buffer.
- [ ] Support `0` and `Home` to go to the beginning of the current line in the request/response buffer.
- [ ] Support `$` and `End` to go to the end of the current line in the request/response buffer.
- [ ] Support `Shift + a` to append text to the end of the current line in the request buffer.
- [ ] Support `Shift + i` to insert text at the beginning of the current line in the request buffer.
- [ ] Support `Ctrl + r` to refresh the current pane.
- [ ] Support `Ctrl + l` to clear the current pane.
- [ ] Support `v` to enter visual mode in the request/response buffer.
- [ ] Support `y` to copy the selected text in the request/response buffer to the clipboard.
- [ ] Support `yy` to copy the current line in the request/response buffer to the clipboard.
- [ ] Support `dd` to cut the current line in the request/response buffer to the clipboard.
- [ ] Support `x` to delete the current character in the request buffer.
- [ ] Support `p` to paste the text in the clipboard in the request buffer.
- [ ] Support `Shift + p` to paste the copied line before the current line in the request buffer.
- [ ] Support `Shift + d` to cut the current character to the end of the line in the request buffer.
- [ ] Support syntax highlighting for HTTP requests in the request buffer.
- [ ] Support `Ctrl + j` to expand the response pane for one line. It goes up to the request pane shrinks to three lines.
- [ ] Support `Ctrl + k` to shrink the response pane for one line. It goes down to the response pane shrinks to three lines.
- [ ] Support `:r` to show/hide the response pane.
- [ ] Optimize memory usage for large response content (>10MB). Implement lazy display cache building and virtual scrolling to prevent memory duplication in display cache.
- [ ] Implement streaming/chunked response handling for very large HTTP responses to avoid loading entire content into memory.
- [ ] Fix background scrolling issue - still occurring despite terminal configuration and alternate screen buffer setup.
- [ ] Print details of the request and response in the beginning of the response pane, when the verbose mode is enabled by `-v` command args. The format is detailed in the monolithic version of the code in the main.rs in the `master` branch (I believe), namely `print_request` and `print_response`.

## Done

- [x] Support `G` to go to the bottom of the current pane.
- [x] Support `gg` to go to the top of the current pane.
- [x] Rename command terminology for clarity and alignment to the Vim terminologies: h/j/k/l as "navigation commands", i/a/A as "editing commands", :q/:set as "ex commands", Ctrl+C as "application commands"
- [x] Revert the HTTP status icon to the original design in the MVC code.
- [x] Make a list of supported commands and their descriptions in docs/COMMANDS.md.
- [x] Restore the Cucumber test capability. It's in the `test_archived` directory. Put it back as integration test and calibrate it to the current codebase.
- [x] Show line number 1 in the request pane at all the time.
- [x] Fix the position indicator in the status bar adjustment. I want to remove `|` between the pane and position indicators as they are related. The new look will be like `([http status code] [message] | [turn around time] |) [mode] | [pane] [line:column])`. The position indicator should be aligned to the right of the status bar.
- [x] Allow ex commands in response pane.
- [x] Refactor the view_model.rs. It's too large and too monolithic. Break it down into smaller components for better maintainability.
- [x] Reduce flickering. It's happening all over. The scope of rendering must be limited to the area that has changed, not the whole screen. Hide cursor before the screen refresh and restore it after the refresh to avoid flickering.
- [x] Support horizontal scrolling in the request/response pane. Use Shift+Left/Right or Ctrl+Left/Right arrow keys to scroll horizontally by 5 characters. Cursor automatically scrolls into view when moving beyond visible area.
- [x] Change cursor shape when switching between normal (block), command (underline), and insert (bar) modes.
- [x] Hide cursor when it is switched to the command mode. Restore the cursor when it is switched back to the normal mode.
- [x] Support word wrap by `:set wrap` and `:set nowrap` in both request and response pane. The wrap setting is effective in both simultaneously. Allow navigation keys to scrolling up and down the pages. Update all commands to respect the word wrap setting.
- [x] Restore the logical line number in the request/response buffer. Minimal width for the line number should be 3. The tilda should be placed in the first column. Refer to the MVC code for the details.
- [x] Support `:q` and `:q!` to quit the application.
- [x] Support `I` command to insert text at the current cursor position in the request buffer.
- [x] Support `a` command to insert text next to the current cursor position in the request buffer.
- [x] Support `Delete` to delete the current character in the request buffer.
- [x] Stop using different color in the current line and status bar. Use the same color as the rest of the text in the request/response buffer. Refer to the MVC code for the details.
- [x] Align The mode, pane, and position indicators in the status bar to the right.
- [x] Restore the HTTP status code with signal light, message, turn around time in the status bar. Show it before the mode indicator. Refer to the MVC code for the details.
- [x] The last HTTP status code, message, and turn around time must be retained and aligned to the right in the status bar.
- [x] initial width for the line number must be 3. The tilda should be placed in the first column.
- [x] src/repl/command.rs can be renamed as commands.rs and we can then migrate mod declaration from repl.rs to this.
- [x] integration tests must incorporate the check for the screen refresh. This could be done by a mock framework that hooks the invocation of a refresh method and maintain/check its call count.
- [x] SwitchPaneCommand in the movement.rs must be pushed out to window.rs.
- [x] Fix the flicking issue when the cursor is moved in the request pane. It only happens in the request pane, regardless of the response pane being shown or not.
- [x] Hide the cursor when it is switched to the command mode. Restore the cursor when it is switched back to the normal mode.
- [x] Make arrow keys work in the request/response pane, regardless of the mode.
