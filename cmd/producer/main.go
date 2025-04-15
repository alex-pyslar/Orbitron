package main

import (
	"fmt"
	"log"
	"time"

	"go-projects/pkg/config"
	"go-projects/pkg/kafka"
)

func main() {
	cfg := config.LoadConfig()

	producer, err := kafka.NewProducer(cfg.KafkaBrokers)
	if err != nil {
		log.Fatalf("Ошибка создания продюсера: %v", err)
	}
	defer producer.Close()

	for i := 0; ; i++ {
		msg := fmt.Sprintf("Сообщение #%d от продюсера", i)
		partition, offset, err := producer.SendMessage(cfg.KafkaTopic, msg)
		if err != nil {
			log.Printf("Ошибка: %v", err)
		} else {
			fmt.Printf("Отправлено: partition=%d, offset=%d, msg=%s\n", partition, offset, msg)
		}
		time.Sleep(2 * time.Second)
	}
}
