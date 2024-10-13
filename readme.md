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
