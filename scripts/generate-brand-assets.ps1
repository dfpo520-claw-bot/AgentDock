Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName System.Drawing

$repoRoot = Split-Path -Parent $PSScriptRoot

function Resolve-RepoPath {
    param([Parameter(Mandatory)][string]$Path)

    return Join-Path $repoRoot $Path
}

function Ensure-ParentDirectory {
    param([Parameter(Mandatory)][string]$Path)

    $parent = Split-Path -Parent $Path
    if ($parent -and -not (Test-Path $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }
}

function New-RoundedRectanglePath {
    param(
        [Parameter(Mandatory)][float]$X,
        [Parameter(Mandatory)][float]$Y,
        [Parameter(Mandatory)][float]$Width,
        [Parameter(Mandatory)][float]$Height,
        [Parameter(Mandatory)][float]$Radius
    )

    $diameter = $Radius * 2
    $path = [System.Drawing.Drawing2D.GraphicsPath]::new()
    $path.AddArc($X, $Y, $diameter, $diameter, 180, 90)
    $path.AddArc($X + $Width - $diameter, $Y, $diameter, $diameter, 270, 90)
    $path.AddArc($X + $Width - $diameter, $Y + $Height - $diameter, $diameter, $diameter, 0, 90)
    $path.AddArc($X, $Y + $Height - $diameter, $diameter, $diameter, 90, 90)
    $path.CloseFigure()

    return $path
}

function Set-HighQualityRendering {
    param([Parameter(Mandatory)][System.Drawing.Graphics]$Graphics)

    $Graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $Graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $Graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $Graphics.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::ClearTypeGridFit
}

function Draw-AgentDockMark {
    param(
        [Parameter(Mandatory)][System.Drawing.Graphics]$Graphics,
        [Parameter(Mandatory)][float]$X,
        [Parameter(Mandatory)][float]$Y,
        [Parameter(Mandatory)][float]$Size
    )

    $navy = [System.Drawing.Color]::FromArgb(255, 18, 34, 58)
    $blue = [System.Drawing.Color]::FromArgb(255, 51, 105, 255)
    $sky = [System.Drawing.Color]::FromArgb(255, 103, 177, 255)
    $green = [System.Drawing.Color]::FromArgb(255, 39, 215, 136)
    $white = [System.Drawing.Color]::FromArgb(255, 244, 249, 255)

    $outer = New-RoundedRectanglePath -X $X -Y $Y -Width $Size -Height $Size -Radius ($Size * 0.23)
    $outerBrush = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
        [System.Drawing.PointF]::new($X, $Y),
        [System.Drawing.PointF]::new($X + $Size, $Y + $Size),
        $navy,
        $blue
    )

    $Graphics.FillPath($outerBrush, $outer)
    $outerBrush.Dispose()

    $dockPen = [System.Drawing.Pen]::new($white, $Size * 0.086)
    $dockPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $dockPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $dockPen.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round

    $path = [System.Drawing.Drawing2D.GraphicsPath]::new()
    $path.StartFigure()
    $path.AddLines([System.Drawing.PointF[]]@(
        [System.Drawing.PointF]::new($X + $Size * 0.29, $Y + $Size * 0.68),
        [System.Drawing.PointF]::new($X + $Size * 0.49, $Y + $Size * 0.31),
        [System.Drawing.PointF]::new($X + $Size * 0.69, $Y + $Size * 0.68)
    ))
    $Graphics.DrawPath($dockPen, $path)
    $path.Dispose()

    $midPen = [System.Drawing.Pen]::new($sky, $Size * 0.055)
    $midPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $midPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $Graphics.DrawLine($midPen, $X + $Size * 0.37, $Y + $Size * 0.55, $X + $Size * 0.61, $Y + $Size * 0.55)
    $midPen.Dispose()

    $greenBrush = [System.Drawing.SolidBrush]::new($green)
    $Graphics.FillEllipse($greenBrush, $X + $Size * 0.67, $Y + $Size * 0.17, $Size * 0.16, $Size * 0.16)
    $greenBrush.Dispose()

    $dockPen.Dispose()
    $outer.Dispose()
}

function Save-Png {
    param(
        [Parameter(Mandatory)][System.Drawing.Bitmap]$Bitmap,
        [Parameter(Mandatory)][string]$Path
    )

    Ensure-ParentDirectory -Path $Path
    $Bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
}

function New-SquareAsset {
    param(
        [Parameter(Mandatory)][int]$Size,
        [Parameter(Mandatory)][string]$OutputPath
    )

    $bitmap = [System.Drawing.Bitmap]::new($Size, $Size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        Set-HighQualityRendering -Graphics $graphics
        $graphics.Clear([System.Drawing.Color]::Transparent)

        $markSize = $Size * 0.76
        $markX = ($Size - $markSize) / 2
        $markY = ($Size - $markSize) / 2
        Draw-AgentDockMark -Graphics $graphics -X $markX -Y $markY -Size $markSize

        Save-Png -Bitmap $bitmap -Path (Resolve-RepoPath $OutputPath)
    }
    finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}

function New-BrandAsset {
    param(
        [Parameter(Mandatory)][string]$OutputPath,
        [int]$Width = 960,
        [int]$Height = 280
    )

    $bitmap = [System.Drawing.Bitmap]::new($Width, $Height, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        Set-HighQualityRendering -Graphics $graphics
        $graphics.Clear([System.Drawing.Color]::Transparent)

        $markSize = $Height * 0.70
        $markX = $Height * 0.15
        $markY = ($Height - $markSize) / 2
        Draw-AgentDockMark -Graphics $graphics -X $markX -Y $markY -Size $markSize

        $fontFamily = 'Segoe UI'
        $fontSize = [single]($Height * 0.28)
        $font = [System.Drawing.Font]::new($fontFamily, $fontSize, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
        $textBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(255, 20, 31, 48))
        $format = [System.Drawing.StringFormat]::new()
        $format.Alignment = [System.Drawing.StringAlignment]::Near
        $format.LineAlignment = [System.Drawing.StringAlignment]::Center

        $textX = $markX + $markSize + ($Height * 0.16)
        $textRect = [System.Drawing.RectangleF]::new($textX, 0, $Width - $textX - ($Height * 0.12), $Height)
        $graphics.DrawString('AgentDock', $font, $textBrush, $textRect, $format)

        $accentBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(255, 39, 215, 136))
        $accentHeight = [single]($Height * 0.035)
        $accentWidth = [single]($Height * 0.70)
        $accentX = [single]$textX
        $accentY = [single]($Height * 0.73)
        $accentPath = New-RoundedRectanglePath -X $accentX -Y $accentY -Width $accentWidth -Height $accentHeight -Radius ($accentHeight / 2)
        $graphics.FillPath($accentBrush, $accentPath)

        $accentPath.Dispose()
        $accentBrush.Dispose()
        $format.Dispose()
        $textBrush.Dispose()
        $font.Dispose()

        Save-Png -Bitmap $bitmap -Path (Resolve-RepoPath $OutputPath)
    }
    finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}

New-SquareAsset -Size 1024 -OutputPath 'docs/agentdock-icon.png'
New-SquareAsset -Size 1024 -OutputPath 'src-tauri/icons/1024x1024.png'
New-SquareAsset -Size 256 -OutputPath 'public/images/logo.png'
New-SquareAsset -Size 256 -OutputPath 'public/favicon-source.png'
New-SquareAsset -Size 256 -OutputPath 'docs/logo.png'
New-BrandAsset -OutputPath 'public/images/logo-brand.png'
New-BrandAsset -OutputPath 'docs/agentdock-logo-brand.png'

Write-Output 'Generated AgentDock brand assets.'
