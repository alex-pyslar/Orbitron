package main

import (
	"fmt"
	"log"

	"go-projects/pkg/config"
	"go-projects/pkg/kafka"
)

func main() {
	cfg := config.LoadConfig()

	consumer, err := kafka.NewConsumer(cfg.KafkaBrokers)
	if err != nil {
		log.Fatalf("Ошибка создания консюмера: %v", err)
	}
	defer consumer.Close()

	err = consumer.ConsumeTopic(cfg.KafkaTopic, func(msg string) {
		fmt.Printf("Получено: %s\n", msg)
	})
	if err != nil {
		log.Fatalf("Ошибка чтения темы: %v", err)
	}
}
