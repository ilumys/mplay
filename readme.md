performance is so poor right now, in great need of review

todo:
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

structure:
- player loads with a base directory (~/Music/)
- all supported tracks are loaded from this directory recursively
- tracks are then grouped by album and by albumartist
  optimise- cache discovered metadata and track changes with lock file/ hashes
- select track to play
- track is moved to front of queue or queue is emptied save track
  all playback should use the queue as its source of truth
- metadata displayed in player for current track
- at completion of track, move to next in queue if exists, then proceed to previous point

track struct:
- album
- track num in album (move to album?)
- albumartist
- artist
- duration
- lyrics

for queue, vecdeque<pointer_to_track>

acknowledgements:
- ratatui, as the tui library, and for the documentation provided
- binsider, as a point of reference for tui impl
- symphonia & rodio for audio library capabilities
