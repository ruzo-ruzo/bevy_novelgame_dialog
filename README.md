# bevy_novelgame_dialog
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-v0.16-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![CI](https://github.com/ruzo-ruzo/bevy_novelgame_dialog/actions/workflows/bevy_ci.yml/badge.svg)](https://github.com/ruzo-ruzo/bevy_novelgame_dialog/actions/workflows/bevy_ci.yml)

## Bevy-engine's dialog Plug-in by Text2D

This plugin provides a novel-game-style dialog box for the Bevy engine. You can write scripts in a Markdown-like format to create dialogues.

## Features
- Display dialog boxes for Bevy engine's novel games.
- Write scripts in a Markdown-like syntax.
- Skip functionality enabled via key pressing (continuous skip feature by holding down a key is not yet available).
- Automatically switches to a lower priority font if glyphs are missing, by specifying multiple fonts.
- Branching based on choices is supported.
- Includes sample UI.

## Unsupported features
- Ligatures
- Text wrapping

## Example Script
```markdown
# Choices
## Docent
Open the choices box.[^wait]
* [Fox walking](choice_example.md#Walking)
* [Fox stopping](choice_example.md#Stopping)
* [Fox running](choice_example.md#Running)
* [Close dialog box](choice_example.md#Closing)

# Walking
[^feed]
The fox is walking[^wait]
[^signal(Fox_walk)]
[jump](choice_example.md#Choices)

# Stopping
[^feed]
The fox is stopping[^wait]
[^signal(Fox_stop)]
[jump](choice_example.md#Choices)

# Running
[^feed]
The fox is running[^wait]
[^signal(Fox_run)]
[jump](choice_example.md#Choices)

# Closing
[^feed]
This dialog box is closing[^wait][^close]

[^wait]: Waiting for input  
[^feed]: Force feeding
[^signal(Fox_walk)]: Play fox walking motion  
[^signal(Fox_stop)]: Play fox searching motion  
[^signal(Fox_run)]: Play fox running motion  
[^close]: Close dialog box
```

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
