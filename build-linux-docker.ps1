# Скрипт для сборки Linux версии в Docker
Write-Host "=== Сборка Notes для Linux в Docker ===" -ForegroundColor Green

# Проверяем что Docker запущен
try {
    docker info | Out-Null
    Write-Host "✅ Docker запущен" -ForegroundColor Green
} catch {
    Write-Host "❌ Docker не запущен или недоступен" -ForegroundColor Red
    Write-Host "Запустите Docker Desktop и попробуйте снова" -ForegroundColor Yellow
    exit 1
}

# Создаем папку для Linux релиза
if (!(Test-Path "releases\linux-x64")) {
    New-Item -ItemType Directory -Path "releases\linux-x64" -Force | Out-Null
    Write-Host "📁 Создана папка releases\linux-x64" -ForegroundColor Yellow
}

Write-Host "🔨 Начинаем сборку в Docker контейнере..." -ForegroundColor Cyan

# Собираем Docker образ и извлекаем бинарник
try {
    # Сборка образа
    Write-Host "📦 Создание Docker образа..." -ForegroundColor Yellow
    docker build -f Dockerfile.build -t notes-linux-builder .
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Ошибка при создании Docker образа" -ForegroundColor Red
        exit 1
    }
    
    # Создание временного контейнера и копирование файла
    Write-Host "📂 Извлечение собранного бинарника..." -ForegroundColor Yellow
    $containerId = docker create notes-linux-builder
    docker cp "${containerId}:/output/Notes" "releases\linux-x64\Notes"
    docker rm $containerId | Out-Null
    
    if ($LASTEXITCODE -eq 0 -and (Test-Path "releases\linux-x64\Notes")) {
        $fileSize = (Get-Item "releases\linux-x64\Notes").Length / 1MB
        $fileSizeMB = [math]::Round($fileSize, 2)
        
        Write-Host "✅ Сборка Linux успешна!" -ForegroundColor Green
        Write-Host "📁 Файл: releases\linux-x64\Notes ($fileSizeMB MB)" -ForegroundColor Green
        
        # Показываем информацию о файле
        Write-Host "`n📊 Информация о файле:" -ForegroundColor Cyan
        Write-Host "Бинарник для Linux x64 готов" -ForegroundColor White
        
    } else {
        Write-Host "❌ Ошибка при копировании файла" -ForegroundColor Red
        exit 1
    }
    
} catch {
    Write-Host "❌ Ошибка при сборке: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

Write-Host "`n🎉 Готово! Linux версия собрана в Docker" -ForegroundColor Green
Write-Host "📁 Результат: releases\linux-x64\Notes" -ForegroundColor Cyan

# Очистка Docker образа (опционально)
$cleanup = Read-Host "`nУдалить временный Docker образ? (y/N)"
if ($cleanup -eq "y" -or $cleanup -eq "Y") {
    docker rmi notes-linux-builder | Out-Null
    Write-Host "🧹 Docker образ удален" -ForegroundColor Yellow
} 