todo:
- volume and play-pause support through rodio sink
- display track duration
- refine metadata retrieval and output
  seriously consider a new dependency to retrieve file tags (or do it myself)
- mpris-enable for play-pause and forward-backward control via keybind
- optionally daemonise
- make concurrent? to ensure selecting a track does not block other actions
- use a queue to play tracks and hold metadata, as opposed to spamming append()
- whole bunch of ui fun. yay
  tabs- queue and library?
  displaying track details and metadata- columns for name/artist/duration?
- option to play all from album/artist/library or shuffle

structure:
- player loads with a base directory (~/Music/)
- all supported tracks are loaded from this directory recursively
- tracks are then grouped by album and by albumartist
  requires opening a file buffer to read metadata :(
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
