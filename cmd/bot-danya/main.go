package main

import (
	"encoding/json"
	"log"
	"strconv"

	tgbotapi "github.com/go-telegram-bot-api/telegram-bot-api/v5"
)

func main() {
	// Токен бота
	botToken := "7744181190:AAHfZJrFMLftQjX0deuuvBZRgluM8MPeblc"
	if botToken == "" {
		log.Fatal("BOT_TOKEN not set")
	}

	// ID канала для проверки подписки
	channelID := int64(-1002568086808)

	// ID приватной группы
	privateGroupID := int64(-1002568050664) // Убедитесь, что это актуальный ID группы

	// Создаем новый экземпляр бота
	bot, err := tgbotapi.NewBotAPI(botToken)
	if err != nil {
		log.Fatal(err)
	}

	bot.Debug = true
	log.Printf("Авторизован под аккаунтом %s", bot.Self.UserName)
	log.Println("Бот запущен")

	// Проверяем доступ к группе
	log.Printf("Проверяем доступ бота к группе %d", privateGroupID)
	chatInfo, err := bot.GetChat(tgbotapi.ChatInfoConfig{ChatConfig: tgbotapi.ChatConfig{ChatID: privateGroupID}})
	if err != nil {
		log.Printf("Ошибка доступа к группе %d: %v", privateGroupID, err)
	} else {
		log.Printf("Группа %d существует, название: %s", privateGroupID, chatInfo.Title)
	}

	// Настраиваем обновления
	u := tgbotapi.NewUpdate(0)
	u.Timeout = 60

	updates := bot.GetUpdatesChan(u)

	// Обработка обновлений
	for update := range updates {
		// Обработка команды /start
		if update.Message != nil && update.Message.Text == "/start" {
			log.Printf("Получена команда /start от %s", update.Message.From.UserName)
			keyboard := tgbotapi.NewInlineKeyboardMarkup(
				tgbotapi.NewInlineKeyboardRow(
					tgbotapi.NewInlineKeyboardButtonData("Вступить в группу", "join_group"),
				),
			)
			msg := tgbotapi.NewMessage(update.Message.Chat.ID,
				"Нажми кнопку ниже, чтобы вступить в приватную группу.\nСначала убедись, что ты подписан на наш канал @bot_danya")
			msg.ReplyMarkup = keyboard
			_, err := bot.Send(msg)
			if err != nil {
				log.Printf("Ошибка отправки ответа на /start: %v", err)
			}
			continue
		}

		// Обработка нажатия кнопки
		if update.CallbackQuery != nil {
			callback := update.CallbackQuery
			log.Printf("Получен callback от %s с данными: %s", callback.From.UserName, callback.Data)

			if callback.Data == "join_group" {
				userID := callback.From.ID
				log.Printf("Проверяем подписку для пользователя %d", userID)

				// Проверяем подписку на канал
				member, err := bot.GetChatMember(tgbotapi.GetChatMemberConfig{
					ChatConfigWithUser: tgbotapi.ChatConfigWithUser{
						ChatID: channelID,
						UserID: userID,
					},
				})
				if err != nil {
					log.Printf("Ошибка GetChatMember: %v", err)
					bot.Send(tgbotapi.NewMessage(callback.Message.Chat.ID,
						"Не удалось проверить подписку. Попробуйте позже."))
					continue
				}

				log.Printf("Статус пользователя в канале: %s", member.Status)

				// Проверяем статус подписки
				if member.Status == "member" || member.Status == "administrator" || member.Status == "creator" {
					// Создаём ссылку-приглашение в группу
					log.Printf("Создаём ссылку-приглашение для группы %d", privateGroupID)
					params := tgbotapi.Params{
						"chat_id": strconv.FormatInt(privateGroupID, 10),
					}
					result, err := bot.MakeRequest("exportChatInviteLink", params)
					if err != nil {
						log.Printf("Ошибка при создании ссылки для группы %d: %v", privateGroupID, err)
						bot.Send(tgbotapi.NewMessage(callback.Message.Chat.ID,
							"Не удалось создать ссылку для группы. Обратитесь к администратору."))
						continue
					}
					if !result.Ok {
						log.Printf("Ошибка Telegram API: %s", result.Description)
						bot.Send(tgbotapi.NewMessage(callback.Message.Chat.ID,
							"Ошибка при создании ссылки: "+result.Description))
						continue
					}

					// Декодируем результат как строку
					var inviteLink string
					err = json.Unmarshal(result.Result, &inviteLink)
					if err != nil {
						log.Printf("Ошибка декодирования ссылки: %v", err)
						bot.Send(tgbotapi.NewMessage(callback.Message.Chat.ID,
							"Внутренняя ошибка при создании ссылки."))
						continue
					}

					// Успешное создание ссылки
					log.Printf("Ссылка для группы создана: %s", inviteLink)
					msg := tgbotapi.NewMessage(callback.Message.Chat.ID,
						"Вы подписаны на канал! Перейдите по ссылке, чтобы вступить в группу:\n"+inviteLink)
					_, err = bot.Send(msg)
					if err != nil {
						log.Printf("Ошибка отправки сообщения со ссылкой: %v", err)
					}
				} else {
					log.Printf("Пользователь не подписан, статус: %s", member.Status)
					msg := tgbotapi.NewMessage(callback.Message.Chat.ID,
						"Сначала подпишись на канал @bot_danya и попробуй снова!")
					_, err = bot.Send(msg)
					if err != nil {
						log.Printf("Ошибка отправки сообщения о подписке: %v", err)
					}
				}

				// Подтверждаем callback
				callbackConfig := tgbotapi.NewCallback(callback.ID, "")
				_, err = bot.Send(callbackConfig)
				if err != nil {
					log.Printf("Ошибка подтверждения callback: %v", err)
				}
			}
		}
	}
}
