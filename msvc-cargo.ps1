param(
    [Parameter(Position = 0)]
    [string[]]$CargoArgs
)

function Invoke-CargoWithMsvc {
    <#
    .SYNOPSIS
    在当前 PowerShell 会话中临时注入 MSVC 工具链路径并执行 cargo。

    .DESCRIPTION
    脚本不会永久修改系统环境变量，只影响当前进程。
    若未传入参数，默认执行 cargo build。
    #>
    param(
        [string[]]$ArgsToForward
    )

    $msvcBin = "C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Tools\MSVC\14.29.30133\bin\Hostx64\x64"

    if (-not (Test-Path -LiteralPath $msvcBin)) {
        throw "未找到 MSVC 工具链目录: $msvcBin"
    }

    $env:Path = "$msvcBin;$env:Path"

    if (-not $ArgsToForward -or $ArgsToForward.Count -eq 0) {
        $ArgsToForward = @("build")
    }

    Write-Host "Using MSVC bin: $msvcBin"
    Write-Host ("Running: cargo " + ($ArgsToForward -join " "))
    & cargo @ArgsToForward
    return $LASTEXITCODE
}

$exitCode = Invoke-CargoWithMsvc -ArgsToForward $CargoArgs
exit $exitCode
