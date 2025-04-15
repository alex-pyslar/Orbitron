package kafka

import (
	"context"
	"fmt"
	"sync"

	"github.com/IBM/sarama"
)

// ConsumerConfig holds configuration for the Kafka consumer
type ConsumerConfig struct {
	Brokers []string
	GroupID string
	Version sarama.KafkaVersion
	Config  *sarama.Config
	Topics  []string
	Handler func(context.Context, *sarama.ConsumerMessage) error
}

// Consumer represents a Kafka consumer
type Consumer struct {
	consumerGroup sarama.ConsumerGroup
	config        *ConsumerConfig
	wg            sync.WaitGroup
}

// NewConsumer creates a new Kafka consumer with the given configuration
func NewConsumer(config *ConsumerConfig) (*Consumer, error) {
	if config == nil {
		return nil, fmt.Errorf("config cannot be nil")
	}

	if config.Config == nil {
		config.Config = sarama.NewConfig()
		config.Config.Version = config.Version
		config.Config.Consumer.Return.Errors = true
		config.Config.Consumer.Offsets.Initial = sarama.OffsetNewest
	}

	consumerGroup, err := sarama.NewConsumerGroup(config.Brokers, config.GroupID, config.Config)
	if err != nil {
		return nil, fmt.Errorf("failed to create consumer group: %w", err)
	}

	return &Consumer{
		consumerGroup: consumerGroup,
		config:        config,
	}, nil
}

// Consume starts consuming messages from the configured topics
func (c *Consumer) Consume(ctx context.Context) error {
	handler := &consumerGroupHandler{
		handler: c.config.Handler,
	}

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
			if err := c.consumerGroup.Consume(ctx, c.config.Topics, handler); err != nil {
				return fmt.Errorf("error from consumer: %w", err)
			}
		}
	}
}

// Close gracefully shuts down the consumer
func (c *Consumer) Close() error {
	return c.consumerGroup.Close()
}

// consumerGroupHandler implements sarama.ConsumerGroupHandler
type consumerGroupHandler struct {
	handler func(context.Context, *sarama.ConsumerMessage) error
}

func (h *consumerGroupHandler) Setup(sarama.ConsumerGroupSession) error {
	return nil
}

func (h *consumerGroupHandler) Cleanup(sarama.ConsumerGroupSession) error {
	return nil
}

func (h *consumerGroupHandler) ConsumeClaim(session sarama.ConsumerGroupSession, claim sarama.ConsumerGroupClaim) error {
	for message := range claim.Messages() {
		if err := h.handler(session.Context(), message); err != nil {
			return fmt.Errorf("error processing message: %w", err)
		}
		session.MarkMessage(message, "")
	}
	return nil
}
