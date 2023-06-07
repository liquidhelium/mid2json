# Midi to RPE(Re: Phigros Edit) chart converter

`$ mid2json --help`
```
Usage: mid2json [OPTIONS] <MIDI_PATH>

Arguments:
  <MIDI_PATH>  Name of the input file

Options:
      --id <TARGET_ID>                     Id of the target chart
      --song-file <SONG_FILE>              Song file referred in the chart
      --background-file <BACKGROUND_FILE>  Background image referred in the chart
  -o, --output <OUTPUT_PATH>               The path of the conversation result
  -s, --seprate <SEPRATION_RATE>           seprate the keys
  -v <SPEED>
  -h, --help                               Print help
  -V, --version                            Print version
```

You can start by just doing this:  
```bash
$ ls
abyss.mid
$ mid2json abyss.mid
$ ls
abyss.mid abyss.json
```
Or, you can just drag the midi file and drop it on this application.   
Note that it won't pack the zip file for you.