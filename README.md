# Ascii video visualizer
## Install
**[Ffmpeg](https://ffmpeg.org/download.html) is mandatory to run this program**

Clone the project :
```sh
git clone https://github.com/dirdr/ascii_video_visualizer && cd ascii_video_visualizer
```
## Run
```sh
cargo run -- --<Mode> --[Path]
```
the **Mode** is mandatory to run the visualizer, choose between *--Individual* and *--Mean*

---
#### Mode flag
**Individual** : Resize the image to match your current terminal size and map each pixel to an ascii character

**Mean** : Group pixel by 'packet' and pick an ascii char based on the mean of this packet

> Note : Mean mode is currently not implemented

---
#### Path flag (choose your video)

You can **run** your own video.
One video is included as an exemple *'Drift.mp4'*
if path flag is not specified, the programm will take this video
by default, but 
you can download your own and put it in the resources folder
and spcified the correct path flag to run your video

---
