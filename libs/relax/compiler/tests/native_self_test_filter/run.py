"""Native self-reexec empty filters succeed; a direct Relax CLI keeps its guard."""
from pathlib import Path
import os,subprocess,sys
root=Path(__file__).resolve().parent
runner=str(Path(sys.argv[1]).resolve())
env=dict(os.environ,CARGO_NET_OFFLINE="true")
expected = "test result: ok. 2 passed; 0 failed; 1 ignored;" if os.name == "posix" else "test result: ok. 1 passed; 0 failed; 0 ignored;"
for command in [
    ["cargo","test","--offline","--release","--no-fail-fast","--lib","--","--test-threads=1"],
    [runner,"test","--quiet","--no-live","native-self-test-filter","--release","--lib","--","--test-threads=1"],
]:
    result=subprocess.run(command,cwd=root,env=env,capture_output=True,text=True)
    assert result.returncode==0 and expected in result.stdout,result.stdout+result.stderr
result=subprocess.run([runner,"test","--quiet","--no-live","native-self-test-filter","--release","--lib","--","--exact","__owned_no_matching_test__"],cwd=root,env=env,capture_output=True,text=True)
assert result.returncode==87,result.stdout+result.stderr
print("native self-reexecution filter semantics and direct-CLI no-tests guard agree")
