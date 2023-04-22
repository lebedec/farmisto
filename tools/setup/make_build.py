import os
import shutil
import subprocess
from datetime import datetime
from subprocess import Popen

from PIL import Image


def change_icon():
    main_directory = os.path.dirname(__file__)
    project_directory = os.path.join(main_directory, '..', '..')
    icon_image = Image.open(os.path.join(project_directory, 'client/resources/icon.png'))
    icon_image.save(os.path.join(project_directory, 'client/resources/farmisto.ico'), format='ico', sizes=[(256, 256)])


def build(rebuild=True):
    main_directory = os.path.dirname(__file__)
    dist_directory = os.path.join(main_directory, 'dist', 'farmisto')
    project_directory = os.path.join(main_directory, '..', '..')

    if rebuild:
        client_rebuild = Popen(
            ['cargo', 'build', '--package', 'client', '--bin', 'client', '--release'],
            cwd=project_directory,
        )
        client_rebuild.wait()

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
        ['target/release/client.exe', 'farmisto.exe'],
        ['target/release/deps/fmod.dll', 'fmod.dll'],
        ['target/release/deps/fmodstudio.dll', 'fmodstudio.dll'],
        ['target/release/deps/SDL2.dll', 'SDL2.dll'],
        ['assets/ai', 'assets/ai'],
        ['assets/audio', 'assets/audio'],
        ['assets/fallback', 'assets/fallback'],
        ['assets/fonts', 'assets/fonts'],
        ['assets/shaders', 'assets/shaders'],
        ['assets/spine', 'assets/spine'],
        ['assets/texture', 'assets/texture'],
        ['assets/text', 'assets/text'],
        ['assets/assets.sqlite', 'assets/assets.sqlite'],
        ['assets/database.sqlite', 'assets/database.sqlite'],
        ['assets/saves', 'assets/saves'],
        ['tools/setup/debug/debug.bat', 'debug.bat'],
        ['tools/setup/debug/farmisto.json', 'farmisto.json'],
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


if __name__ == '__main__':
    # change_icon()
    build()
