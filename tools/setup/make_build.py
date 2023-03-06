import os
import shutil
import subprocess
from datetime import datetime
from subprocess import Popen

if __name__ == '__main__':
    main_directory = os.path.dirname(__file__)
    dist_directory = os.path.join(main_directory, 'dist', 'farmisto')
    project_directory = os.path.join(main_directory, '..', '..')
    git_commit_fetch = Popen(
        ['git', 'rev-parse', '--short', 'HEAD'],
        cwd=project_directory,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT
    )
    commit_hash = git_commit_fetch.stdout.readline().decode('utf-8').strip()
    time_label = datetime.now().strftime('%Y%m%d')
    archive_name = f'farmisto-{time_label}-{commit_hash}'
    archive_path = os.path.join('dist', archive_name)

    if os.path.exists(dist_directory):
        shutil.rmtree(dist_directory)

    files = [
        ['target/release/client.exe', 'client.exe'],
        ['target/release/deps/fmod.dll', 'fmod.dll'],
        ['target/release/deps/fmodstudio.dll', 'fmodstudio.dll'],
        ['assets/audio', 'assets/audio'],
        ['assets/fallback', 'assets/fallback'],
        ['assets/shaders', 'assets/shaders'],
        ['assets/spine', 'assets/spine'],
        ['assets/texture', 'assets/texture'],
        ['assets/assets.sqlite', 'assets/assets.sqlite'],
        ['assets/database.sqlite', 'assets/database.sqlite'],
    ]
    for src, dst in files:
        src = os.path.join(project_directory, src)
        dst = os.path.join(dist_directory, dst)
        if os.path.isdir(src):
            shutil.copytree(src, dst)
        else:
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            shutil.copy(src, dst)
    with open(os.path.join(dist_directory, 'version.txt'), 'w') as version:
        version.write(f'{archive_name}\n')
    shutil.make_archive(archive_path, 'zip', dist_directory)

    print('hello', archive_name)
