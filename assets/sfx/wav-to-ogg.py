import sys
import subprocess

input_file = sys.argv[1]
output_file = sys.argv[2]
subprocess.run(['ffmpeg', '-i', input_file, '-c:a', 'libvorbis', output_file])
