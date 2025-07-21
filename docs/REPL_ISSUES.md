# REPL Problems

## Features

### Fixes

#### Request Pane

##### Normal Mode

1. Scroll does not work sometimes. Not sure what triggers it.
1. Scroll flicks the Response pane
1. `dd`, `x`, `d`, `D` are not working
1. Overall the pane does not represent what the Request buffer has.

#### Response Pane

##### Normal Mode

1. Ctrl + M does not maximize the pane.

##### Visual Mode

1. Visual selection + cursor move causes flickers a LOT.
1. `v` + downward cursor move does not scroll the pane. Upward scroll works.
1. `v` + `gg` does not work.
1. `v` + `G` seems to be working but doesn't scroll to the bottom of the buffer.

### Enhancements (After the above fixes)

1. I want to print tilda (~) for the empty lines like vi.
1. I want to skip space characters in `w` and `b` commands like vi.
1. I want to support `p` and `P` to paste the content of the clipboard in the Request pane.
1. I want `Ctrl` + `R` to refresh the pane (both) so I can make sure what I have in the buffer.
1. I want to support Tab. The default tab stop is 4 and to be configurable by `set tabstop=4`.
1. I want to be able to use `Ctrl` + `C` to cancel the request.
1. I want to support `Ctrl` + `Enter` to send the request.
1. I want the font color when the pane is not focused one or two step lighter.
1. I want to ring a short beep when the response pane is refreshed by the new request. Only when it took more than 5 second to process the request.
1. I want to refrain from submitting a request when the previous request is still being processed.
1. I want to support `:` + number to jump to the line number in the pane.
