package config

import (
	"os"
	"strings"
)

// Config содержит настройки приложения
type Config struct {
	KafkaBrokers []string // Список брокеров Kafka
	KafkaTopic   string   // Топик для обмена сообщениями
}

// LoadConfig загружает конфигурацию из переменных окружения
func LoadConfig() *Config {
	return &Config{
		KafkaBrokers: getEnvAsSlice("KAFKA_BROKERS", []string{"localhost:9092"}, ","),
		KafkaTopic:   getEnv("KAFKA_TOPIC", "test-topic"),
	}
}

// getEnv возвращает значение переменной окружения или дефолтное значение
func getEnv(key, defaultValue string) string {
	if value, exists := os.LookupEnv(key); exists {
		return value
	}
	return defaultValue
}

// getEnvAsSlice возвращает значение переменной окружения как срез строк
func getEnvAsSlice(key string, defaultValue []string, separator string) []string {
	if value, exists := os.LookupEnv(key); exists {
		return strings.Split(value, separator)
	}
	return defaultValue
}
