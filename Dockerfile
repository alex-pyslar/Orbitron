# Используем легкий базовый образ с Linux
FROM ubuntu:22.04

# Устанавливаем зависимости, если нужно (LLVM, GraalVM и т.д.)
RUN apt-get update && \
    apt-get install -y clang default-jdk curl && \
    rm -rf /var/lib/apt/lists/*

# Создаём рабочую директорию
WORKDIR /orbitron

# Копируем бинарник компилятора в контейнер
COPY target/release/orbitron /usr/local/bin/orbitron

# Делаем его исполняемым
RUN chmod +x /usr/local/bin/orbitron

# По умолчанию запускаем CLI компилятора
ENTRYPOINT ["orbitron"]