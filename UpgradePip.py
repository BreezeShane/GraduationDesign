import sys
import subprocess

sys.setrecursionlimit(10000)
subprocess.check_call([sys.executable, "-m", "pip", "install", "--upgrade", "pip"])