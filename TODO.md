# TODO

## Features to add

### Before Commit

### Now

* WHen a shader fails to verify (generation) then like dont use it.
* Looks like one cell is always erroring, why
* Make a test LLM provider which just uses default shaders.
* It looks like shaders states are rebuilt every frame when there is an error.  See if that is true when there is no error and fix both.
* Settings dialog not implemented, see if this is in next phase.
* Save dialog not implemented correctly, it should save the entire shader Specimen lineage, check if this is in next phase.
* TODO NOW pass
* Make shader treat the box it is in as the whole "screen"
* Mouse not taken into account
* Error handling and over requesting.
* use default shader and clean stuff up.
* Is audio being actually passed yet? Check tasks if it is next before trying to fix.

### Features

* Add CLAP cli along with help texts etc (eg where are the config.toml files read from in what priority), and config override.
* Add visual feedback for shader generation progress (including messages in an expandable/optional log area)
* Ability to load a shader by name/uuid/path/etc in fullscreen mode just to vibe with it.
* Ability to modify prompt as specimens are generated to help guide.
* Server mode with web UI
* Auth for server
* Hosting for real server
* Ability to browse other users' generated shaders with good UI to see the evolution and pick up from a spot in evolution yourself.
* Ability to share generated shaders with link.
* Ability to charge users and provide LLM key for them when running as a webserver.
* Ability to charge users and provide LLM for them when running for desktop mode.

* Multilang support

* Other kinds of parent mutation
    // OperatorSwap,
    // ConstantTweak,
    // ColorChannelSwap,
    // BlockInsert,
    // BlockRemove,
    // Crossover,
