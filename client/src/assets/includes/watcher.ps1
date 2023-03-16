$watcher = New-Object System.IO.FileSystemWatcher
$watcher.Path = "assets"
$watcher.Filter = "*.*"
$watcher.IncludeSubdirectories = $true
$watcher.EnableRaisingEvents = $true

$generic = {
    $path = $Event.SourceEventArgs.FullPath
    $changeType = $Event.SourceEventArgs.ChangeType
    Write-Host "$changeType`:$path"
}

$renamed = {
    $old = $Event.SourceEventArgs.OldFullPath
    $path = $Event.SourceEventArgs.FullPath
    Write-Host "Deleted:$old"
    Write-Host "Created:$path"
}

Register-ObjectEvent $watcher "Created" -Action $generic | out-null
Register-ObjectEvent $watcher "Changed" -Action $generic | out-null
Register-ObjectEvent $watcher "Deleted" -Action $generic | out-null
Register-ObjectEvent $watcher "Renamed" -Action $renamed | out-null

while ($true) {sleep 0.1}