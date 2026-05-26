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

    $slate0 = [System.Drawing.Color]::FromArgb(255, 15, 23, 32)
    $slate1 = [System.Drawing.Color]::FromArgb(255, 24, 34, 48)
    $teal = [System.Drawing.Color]::FromArgb(255, 45, 212, 191)
    $blue = [System.Drawing.Color]::FromArgb(255, 96, 165, 250)
    $white = [System.Drawing.Color]::FromArgb(255, 237, 244, 251)

    $outer = New-RoundedRectanglePath -X $X -Y $Y -Width $Size -Height $Size -Radius ($Size * 0.23)
    $outerBrush = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
        [System.Drawing.PointF]::new($X, $Y),
        [System.Drawing.PointF]::new($X + $Size, $Y + $Size),
        $slate0,
        $slate1
    )

    $Graphics.FillPath($outerBrush, $outer)
    $outerBrush.Dispose()

    $borderPen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(90, 237, 244, 251), [Math]::Max(1.0, $Size * 0.012))
    $Graphics.DrawPath($borderPen, $outer)
    $borderPen.Dispose()

    $ringRect = [System.Drawing.RectangleF]::new($X + $Size * 0.18, $Y + $Size * 0.18, $Size * 0.64, $Size * 0.64)
    $ringPen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(150, $teal), $Size * 0.048)
    $ringPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $ringPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $Graphics.DrawArc($ringPen, $ringRect, 204, 270)
    $ringPen.Dispose()

    $dockPen = [System.Drawing.Pen]::new($blue, $Size * 0.042)
    $dockPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $dockPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $Graphics.DrawLine($dockPen, $X + $Size * 0.28, $Y + $Size * 0.72, $X + $Size * 0.72, $Y + $Size * 0.72)
    $Graphics.DrawLine($dockPen, $X + $Size * 0.50, $Y + $Size * 0.58, $X + $Size * 0.50, $Y + $Size * 0.72)
    $dockPen.Dispose()

    $fontSize = [single]($Size * 0.30)
    $font = [System.Drawing.Font]::new('Segoe UI', $fontSize, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
    $textBrush = [System.Drawing.SolidBrush]::new($white)
    $format = [System.Drawing.StringFormat]::new()
    $format.Alignment = [System.Drawing.StringAlignment]::Center
    $format.LineAlignment = [System.Drawing.StringAlignment]::Center
    $textRect = [System.Drawing.RectangleF]::new($X + $Size * 0.18, $Y + $Size * 0.25, $Size * 0.64, $Size * 0.38)
    $Graphics.DrawString('AD', $font, $textBrush, $textRect, $format)

    $nodeBrush = [System.Drawing.SolidBrush]::new($teal)
    $Graphics.FillEllipse($nodeBrush, $X + $Size * 0.69, $Y + $Size * 0.18, $Size * 0.13, $Size * 0.13)
    $nodeBrush.Dispose()

    $format.Dispose()
    $textBrush.Dispose()
    $font.Dispose()
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

function Save-TextUtf8NoBom {
    param(
        [Parameter(Mandatory)][string]$Text,
        [Parameter(Mandatory)][string]$Path
    )

    Ensure-ParentDirectory -Path $Path
    $encoding = [System.Text.UTF8Encoding]::new($false)
    [System.IO.File]::WriteAllText($Path, $Text, $encoding)
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
        $textBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(255, 23, 32, 51))
        $format = [System.Drawing.StringFormat]::new()
        $format.Alignment = [System.Drawing.StringAlignment]::Near
        $format.LineAlignment = [System.Drawing.StringAlignment]::Center

        $textX = $markX + $markSize + ($Height * 0.16)
        $textRect = [System.Drawing.RectangleF]::new($textX, 0, $Width - $textX - ($Height * 0.12), $Height)
        $graphics.DrawString('AgentDock', $font, $textBrush, $textRect, $format)

        $accentBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(255, 15, 118, 110))
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

function New-SvgAsset {
    param([Parameter(Mandatory)][string]$OutputPath)

    $svg = @'
<?xml version="1.0" encoding="UTF-8"?>
<svg width="512" height="512" viewBox="0 0 512 512" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="agentdock-bg" x1="61" y1="61" x2="451" y2="451" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="#0f1720"/>
      <stop offset="1" stop-color="#182230"/>
    </linearGradient>
  </defs>
  <rect x="61" y="61" width="390" height="390" rx="90" fill="url(#agentdock-bg)" stroke="#edf4fb" stroke-opacity="0.35" stroke-width="6"/>
  <path d="M142 205 A124 124 0 1 1 205 369" fill="none" stroke="#2dd4bf" stroke-opacity="0.59" stroke-width="19" stroke-linecap="round"/>
  <path d="M170 341 H342 M256 287 V341" fill="none" stroke="#60a5fa" stroke-width="16" stroke-linecap="round"/>
  <text x="256" y="255" fill="#edf4fb" font-family="Segoe UI, Arial, sans-serif" font-size="117" font-weight="700" text-anchor="middle" dominant-baseline="middle">AD</text>
  <circle cx="330" cy="132" r="25" fill="#2dd4bf"/>
</svg>
'@

    Save-TextUtf8NoBom -Text $svg -Path (Resolve-RepoPath $OutputPath)
}

New-SquareAsset -Size 1024 -OutputPath 'docs/agentdock-icon.png'
New-SquareAsset -Size 256 -OutputPath 'public/images/logo.png'
New-SquareAsset -Size 256 -OutputPath 'public/favicon-source.png'
New-SquareAsset -Size 256 -OutputPath 'docs/logo.png'
New-BrandAsset -OutputPath 'public/images/logo-brand.png'
New-BrandAsset -OutputPath 'docs/agentdock-logo-brand.png'
New-SvgAsset -OutputPath 'src-tauri/icons/icon.svg'

New-SquareAsset -Size 16 -OutputPath 'src-tauri/icons/16x16.png'
New-SquareAsset -Size 32 -OutputPath 'src-tauri/icons/16x16@2x.png'
New-SquareAsset -Size 32 -OutputPath 'src-tauri/icons/32x32.png'
New-SquareAsset -Size 64 -OutputPath 'src-tauri/icons/32x32@2x.png'
New-SquareAsset -Size 64 -OutputPath 'src-tauri/icons/64x64.png'
New-SquareAsset -Size 128 -OutputPath 'src-tauri/icons/64x64@2x.png'
New-SquareAsset -Size 128 -OutputPath 'src-tauri/icons/128x128.png'
New-SquareAsset -Size 256 -OutputPath 'src-tauri/icons/128x128@2x.png'
New-SquareAsset -Size 256 -OutputPath 'src-tauri/icons/256x256.png'
New-SquareAsset -Size 512 -OutputPath 'src-tauri/icons/256x256@2x.png'
New-SquareAsset -Size 512 -OutputPath 'src-tauri/icons/512x512.png'
New-SquareAsset -Size 1024 -OutputPath 'src-tauri/icons/1024x1024.png'
New-SquareAsset -Size 512 -OutputPath 'src-tauri/icons/icon.png'

New-SquareAsset -Size 16 -OutputPath 'src-tauri/icons/icon.iconset/icon_16x16.png'
New-SquareAsset -Size 32 -OutputPath 'src-tauri/icons/icon.iconset/icon_16x16@2x.png'
New-SquareAsset -Size 32 -OutputPath 'src-tauri/icons/icon.iconset/icon_32x32.png'
New-SquareAsset -Size 64 -OutputPath 'src-tauri/icons/icon.iconset/icon_32x32@2x.png'
New-SquareAsset -Size 128 -OutputPath 'src-tauri/icons/icon.iconset/icon_128x128.png'
New-SquareAsset -Size 256 -OutputPath 'src-tauri/icons/icon.iconset/icon_128x128@2x.png'
New-SquareAsset -Size 256 -OutputPath 'src-tauri/icons/icon.iconset/icon_256x256.png'
New-SquareAsset -Size 512 -OutputPath 'src-tauri/icons/icon.iconset/icon_256x256@2x.png'
New-SquareAsset -Size 512 -OutputPath 'src-tauri/icons/icon.iconset/icon_512x512.png'
New-SquareAsset -Size 1024 -OutputPath 'src-tauri/icons/icon.iconset/icon_512x512@2x.png'

Write-Output 'Generated AgentDock brand assets.'
