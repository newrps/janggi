$me = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectName = Split-Path -Leaf $me
& "C:\git\deploy-all.ps1" $projectName @args
