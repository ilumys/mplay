todo:
- volume and play-pause support through rodio sink
- display track duration
- refine metadata retrieval and output
  seriously consider a new dependency to retrieve file tags (or do it myself)
- mpris-enable for play-pause and forward-backward control via keybind
- optionally daemonise

remember: sink audio uses a different thread!
using `sleep_until_end` keeps the main thread alive until this other thread finishes
so if some other process is keeping the thread alive, `sleep_until_end` is unnecessary
