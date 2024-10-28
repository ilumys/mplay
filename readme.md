# About

Presently a rather simple terminal-based music player, loading tracks from a directory, sorting by tag, and playing on select.

# Todo

- display track duration
- refine metadata retrieval and output
- mpris-enable for play-pause and forward-backward control via keybind
- optionally daemonise
- make concurrent? to ensure selecting a track does not block other actions
- use a queue to play tracks and hold metadata, as opposed to spamming append()
- whole bunch of ui fun. set context for which 'tab' is selected
- option to play all from album/artist/library or shuffle
- intelligent library compilation i.e., detect artist/album/title from directory hierarchy

aspirational:
- replace rodio with custom sink build on top of symphonia

# Acknowledgements

- ratatui, as the tui library, and for the documentation provided
- binsider, as a point of reference for tui impl
- symphonia & rodio for audio library capabilities
