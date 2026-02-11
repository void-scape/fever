import sys
import subprocess
from pathlib import Path

directory = Path(sys.argv[1])

for wav_file in directory.rglob('*.wav'):
    ogg_file = wav_file.with_suffix('.ogg')
    subprocess.run(['ffmpeg', '-i', str(wav_file), '-c:a', 'libvorbis', str(ogg_file)])
    wav_file.unlink()
