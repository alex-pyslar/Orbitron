package kafka

import (
	"github.com/IBM/sarama"
	"log"
)

type Producer struct {
	syncProducer sarama.SyncProducer
}

func NewProducer(brokers []string) (*Producer, error) {
	config := sarama.NewConfig()
	config.Producer.Return.Successes = true
	config.Producer.RequiredAcks = sarama.WaitForAll

	producer, err := sarama.NewSyncProducer(brokers, config)
	if err != nil {
		return nil, err
	}
	return &Producer{syncProducer: producer}, nil
}

func (p *Producer) SendMessage(topic, message string) (int32, int64, error) {
	msg := &sarama.ProducerMessage{
		Topic: topic,
		Value: sarama.StringEncoder(message),
	}
	partition, offset, err := p.syncProducer.SendMessage(msg)
	if err != nil {
		log.Printf("Ошибка отправки: %v", err)
	}
	return partition, offset, err
}

func (p *Producer) Close() {
	p.syncProducer.Close()
}
