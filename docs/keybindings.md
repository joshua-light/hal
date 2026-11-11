# Keybindings reference

Every key hal binds, per tab and per popup, in both key schemes. The status
bar shows the common ones for the current view.

## Key schemes

| Scheme | In short |
|---|---|
| IDE (default) | Arrows move. `Ctrl+N` new, `Ctrl+T` find, `Ctrl+P` open a folder, `F2` rename, `Del` delete, `Ctrl+Z` undo. Other actions keep a letter. |
| Vim | `hjkl` move, `J`/`K` switch tabs, `/` find, `x` delete, `u` undo. |

Switch in the settings window (`,`, Key scheme), or set `[ui].keys = "vim"` in
`~/.hal/config.toml`. Under IDE, `j` and `k` do nothing but say how to
turn the Vim keys on.

Keys a keyboard may lack have a second key under IDE: `Ins` for `Ctrl+N`,
`Backspace` for `Del`, `r` for `F2` and `F5`, `e` for `F2` on an agent's
setting. Where the Vim column is empty, the key is the same in both.

Text inputs and type-to-filter popups take the same keys in both schemes.

`Cmd` counts as `Ctrl` where the terminal forwards it: `Cmd+P` is `Ctrl+P`.

Keys are read by position, so they work with the Ukrainian or Russian
layout switched on: `о` is `j`, `Л` is `K`, `б` is `,`. Text fields (names,
filters, the pane) keep the letters you type. Punctuation is read as typed,
so `/` and `.` need the English layout.

## Rebinding keys

Any key in the tables below can change. Add a `[keymap.<place>]` table to
`~/.hal/config.toml`, naming the action and its new keys, then restart hal:

```toml
[keymap.sessions]
new_in = ["ctrl+p", "o"]   # two keys
close = "x"                # one key
shell = []                 # no key

[keymap.global]
quit = "ctrl+q"
```

`hal keys` prints every place, action and current key in this form, so
copy the line you want and edit it. `hal keys vim` shows the other scheme.

| Place | Where its keys work |
|---|---|
| `global` | Every tab |
| `sessions`, `board`, `agents`, `metrics` | The Sessions, Tasks, Agents and Metrics tabs |
| `task_info`, `agent_page` | The task info popup; an agent's page |
| `settings` | The settings window |
| `popup`, `live_tail` | Session info; the transcript |
| `list_picker` | The agent, respawn and task-kind pickers |
| `folders` | The folder picker |

Rules:

- An override replaces all keys of its action, under whichever scheme is on.
- Your key wins: any other action in that place that had it loses it. A
  `global` key is also taken from the four tabs.
- Hints in the status bar and popup footers show the new keys.
- A key is a character (`P` is `Shift+P`) or one of `enter`, `esc`, `tab`,
  `space`, `bksp`, `del`, `ins`, `up`, `down`, `left`, `right`, `pgup`,
  `pgdn`, `home`, `end`, `f1`–`f12`, with `ctrl+`, `alt+` or `shift+` in
  front. `cmd+` means `ctrl+`.
- Outside the kitty keyboard protocol a terminal cannot send
  `Ctrl+Shift+letter`, and sends `Ctrl+I`, `Ctrl+M` and `Ctrl+H` as `Tab`,
  `Enter` and `Backspace`.
- A mistyped place, action or key is skipped and named in the status bar at
  startup; `hal keys` lists them all.

Not rebindable: text inputs, type-to-filter popups, the embedded pane,
`Ctrl+L` and agent hotkeys (`[agents.<id>].hotkey`).

## Global

These work on every tab when no popup is open. `Ctrl+L` also works in popups.

| IDE | Vim | Action |
|---|---|---|
| `Tab` / `Ctrl+Tab` | `Tab` / `K` | Next tab |
| `Shift+Tab` / `Ctrl+Shift+Tab` | `Shift+Tab` / `J` | Previous tab |
| `q` / `Ctrl+Q` | `q` | Quit |
| `,` | | Settings: keys, layout, theme and colours |
| `Ctrl+L` | | Repaint the screen |

`Ctrl+Tab` needs a terminal that speaks the kitty keyboard protocol.

Tabs run Tasks → Sessions → Agents → Metrics; hal opens on Sessions.
Agents shows once `~/.hal/agents/` exists, unless
`[harness].show_tab = false`.

## Tasks

| IDE | Vim | Action |
|---|---|---|
| `←` `→` | `h` `l` / `←` `→` | Previous or next column |
| `↑` `↓` | `k` `j` / `↑` `↓` | Previous or next card |
| `Shift+←` / `Shift+→` | `H` / `L` | Move the card one column left or right. Skips Planning: moving a Planning card right does not approve the plan (`Space` does); moving left from Done reopens into Review |
| `Ctrl+N` / `Ins` | `a` / `n` | Add a task |
| `F2` / `r` | `r` | Rename the card |
| `t` | | Edit its tags |
| `T` | | Pick its kind (`[tasks].kinds`) |
| `1`–`4` | | Set priority P1–P4 |
| `s` | | Assign an agent: pick a folder, spawn a planning session there |
| `S` | | Assign an agent in your home folder |
| `Enter` | `Enter` / `f` | Attach the card's agent; resume it if its session died |
| `Space` | | Planning card: approve the plan. Otherwise: toggle Done |
| `v` | | Task Info: prompt, notes and attachments |
| `A` | | Attach a file path or URL |
| `Ctrl+V` / `p` | `p` | Attach the clipboard text as a note. Use `p` where the terminal keeps `Ctrl+V` for its own paste |
| `Ctrl+F` / `/` | `/` | Filter cards by text or `#tag` |
| `Esc` | | Clear the filter |
| `Del` / `Backspace` | `x` | Delete the card (archived); its agent session keeps running |
| `c` | | Clear the Done column (archived) |
| `Ctrl+Z` | `u` | Undo the last delete or clear |

## Sessions

| IDE | Vim | Action |
|---|---|---|
| arrows | `j` `k` `l` / arrows | Move (`h` is handoff; `←` moves left) |
| `Enter` | `Enter` / `f` | Type into the session's pane (docked in the split layout, a popup elsewhere), focus its window, or resume an inactive session |
| `i` | | Session info |
| `v` | | Cycle the layout: split (list beside the selected session's live pane), full-width list, card grid |
| `H` | | Show or hide inactive sessions |
| `Ctrl+T` / `/` | `/` | Find any session on disk, however old, and reopen it |
| `Ctrl+N` / `Ins` | `n` | New session with the default agent, in this session's folder |
| `N` | | Pick a model (and agent), then start a session in this folder |
| `A` | | Choose the default agent until restart |
| `Ctrl+P` | | Pick a folder, start a session there |
| `M` | | Pick a bookmarked folder, start a session there |
| `h` | | Mark the session for a handoff, or drop the mark: the next session you start opens with its last reply, named after it: `Plan` → `Plan (2)` → `Plan (3)` |
| `R` | | Continue the session on another subscription account |
| `F2` / `r` | `r` | Rename the session. The name a new session asks for at boot takes you into it on Enter |
| `L` | | Link the session to a task, or unlink it |
| `t` | | Terminal in this session's folder: a shell listed below the project's agents under its own icon. It asks for a name like a new session; Enter opens it |
| `Del` / `Backspace` | `x` | End the session (asks first): kills its tmux session, else closes its window and terminates the agent |
| `Space` | | Put the session on hold, or release it. Held sessions move to the On hold section below every project, keep their real state, and stop counting toward attention; activity does not release them |
| `z` | | Acknowledge: mark the session idle until its next activity |
| `m` | | Go to Metrics |

`[agents.<id>].hotkey` adds a key that starts that agent in the selected
session's folder. It overrides a built-in key it collides with.

## Agents

The agents run down the side; the selected one's page beside them leads
with how it is doing.

| IDE | Vim | Action |
|---|---|---|
| `↑` `↓` | `j` `k` / `↓` `↑` | Move |
| `Enter` / `i` | `Enter` / `f` / `i` | Move the keyboard to the agent's page |
| `Space` | | Turn the agent off or on (on also clears a halt) |
| `p` | | Run now, the way the trigger would: a poll agent polls |
| `Ctrl+N` / `Ins` | `n` | Interactive Claude session in the agent's folder |

On the page:

| IDE | Vim | Action |
|---|---|---|
| `Tab` / `→` | `Tab` / `l` / `→` | Next section |
| `Shift+Tab` / `←` | `Shift+Tab` / `h` / `←` | Previous section |
| `1`–`4` | | Runs, Artifacts, Log, Settings |
| `↑` `↓` | `j` `k` / `↓` `↑` | Move within the section |
| `Enter` / `o` | `Enter` / `f` / `o` | Open the run's transcript or the artifact; change the setting |
| `F2` / `e` | `e` | Type a new value for the setting (`Enter` saves, `Esc` cancels) |
| `Space`, `p` | | As on the list |
| `Ctrl+N` / `Ins` | `n` | As on the list |
| `R` | | Reset the agent's runs, spend and history (the workdir stays) |
| `Esc` / `F1` / `q` | | Back to the list |

## Metrics

| IDE | Vim | Action |
|---|---|---|
| `↑` `↓` | `j` `k` / `↓` `↑` | Select a session |
| `Enter` | | Open its transcript at its peak context |
| `F5` / `r` | `r` | Rerun the analysis |

## Embedded pane

Every key goes to the pane, except:

| Key | Action |
|---|---|
| `F1` | Close the pane; in the split layout, give the keyboard back to the list |
| `Ctrl+Shift+V` | Paste the host clipboard |

## Mouse

hal takes the mouse while a tab is on screen with no popup over it. To
select text, drag with your terminal's bypass modifier held: usually
`Shift`, `Option` in iTerm2.

| Where | Click | Wheel |
|---|---|---|
| Tab strip | Open the tab, with the keyboard on its list | |
| Sidebar (Sessions split, Agents) | Put the cursor on the row and the keyboard on the list | Move the cursor |
| Page beside it | Give the page the keyboard; once the Sessions dock has it, clicks go to its pane | Scroll the page |
| Sessions list and cards | Put the cursor on the session | Move the cursor |
| Tasks board | Put the cursor on the card; on a column's empty space, on that column | Move the cursor in the column |
| Metrics | | Move the cursor |

## Popups

`Esc` cancels every popup. Type-to-filter popups move with `↑`/`↓` or
`Ctrl+J`/`Ctrl+K`/`Ctrl+N`/`Ctrl+P`, because letters go to the filter.
List popups move with `↑`/`↓`, and with `j`/`k` under Vim.

| Popup | IDE | Vim |
|---|---|---|
| Session info | `↑`/`↓` scroll, `Esc`/`q` close | also `j`/`k` scroll |
| Transcript tail | `↑`/`↓` scroll, `End` jump to the end, `Esc`/`q` close | `j`/`k` scroll, `G` jump to the end |
| Close confirmation | `y` close, `n`/`q` cancel | |
| Model picker (`N`) | type to filter, `Tab` next agent, `Enter`/`Space` start | |
| Session finder | type to filter, `Enter` open | |
| Task link picker (`L`) | type to filter, `Enter`/`Space` link | |
| Agent, respawn, kind pickers | `↑`/`↓` move, `Enter`/`Space` choose | also `j`/`k` move |
| Rename, tags | type, `Enter` save; tags split on space or comma | |
| Add task | type, `Tab` switch to the context box, `Enter` add | |
| Attach | type, `Tab` switch between file/URL and note, `Enter` attach | |
| Filter | type, `Enter` keep the filter, `Esc` clear it | |
| Task Info | `↑`/`↓` attachment, `a` attach, `Ctrl+V`/`p` paste note, `Ctrl+C`/`c` copy path, `Enter`/`o` open, `Del`/`Backspace` remove, `PgUp`/`PgDn` scroll, `Esc`/`v`/`q` close | `j`/`k` attachment, `p` paste note, `c` copy path, `o` open, `x` remove |
| Settings (`,`) | `↑`/`↓` move, `←`/`→` change, `Enter`/`Space` flip or type a colour, `Del`/`Backspace`/`d` back to the default, `Esc`/`q`/`,` close | also `j`/`k` move, `h`/`l` change |

### Folder picker

The picker opens on a fuzzy list of places (bookmarks and recent folders), or in Browse mode when there are none.

| Mode | Keys |
|---|---|
| Places | type to filter, `Enter`/`Space` pick, `Tab` browse folders |
| Browse | `Enter`/`→` open folder, `Backspace`/`←` parent, `Space` pick, `.` pick the current folder, `m` bookmark, `c`/`C` create a public/private GitHub repo here, `Tab` places, `Esc`/`q` cancel |
| Bookmarks (`M`) | `Enter`/`Space`/`.` pick, `m` remove bookmark, `Esc`/`q` cancel |

Browse and Bookmarks move with `↑`/`↓`. Vim adds `j`/`k` to move, `l` to
open a folder and `h` for its parent.

GitHub repo name: type, `Tab` toggle public/private, `Enter` create.
