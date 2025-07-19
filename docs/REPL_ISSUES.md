# REPL Problems

## Features

### Enhancements

1. I want to skip space characters in `w` and `b` commands like vi.
1. I want to print tilda (~) for the empty lines like vi.
1. I want the font color when the pane is not focused one or two step lighter.
1. I want to ring a short beep when the response pane is refreshed by the new request. Only when it took more than 5 second to process the request.
1. I want to not cancel a request when the previous request is still being processed.
1. I want to be able to use `Ctrl` + `C` to cancel the request.
1. I want to support `Ctrl` + `Enter` to send the request.

### Request Pane

#### Normal Mode on Request Pane

1. I suspect any of the normal command keys does not refresh the request pane correctly.
1. `x`, `d`, `D` commands do not refresh the pane. Precisely, after typing those keys, switching to the insert mode will refresh the request pane and the changes are reflected.
1. Support `Ctrl` + `R` to refresh the request pane so I can see what I see is what I have in the buffer.

#### Insert Mode on Request Pane

1. Tab key does not work in the insert mode. It should insert a tab character. The default tab stop is 4 and to be configurable by `set tabstop=4`.

### Response Pane

#### Normal Mode on the Response Pane

1. Flickers when I type Ctrl + F while the pane is at the last page.
1. Flickers when I type Ctrl + B while the pane is at the top page. Only at the first attempt; there is no flicker when I type Ctrl + B again.
1. Ctrl + M does not maximize the pane.


#### Visual Mode on the Response Pane

1. Visual selection causes flickers a lot.
1. In Visual Mode on the Response Pane, the selection with scrolling with `j`, `k` etc does not work.
1. `v` + `gg` or `v` + `G` does not work.

### Status line

## Code Refactoring

1. replace "input" with "request" in the codebase.
1. replace "output" with "response" in the codebase.