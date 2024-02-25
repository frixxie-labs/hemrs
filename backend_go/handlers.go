package main

import (
	"log"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/jmoiron/sqlx"
)

type SensorData struct {
	Ts          time.Time `db:"ts"`
	Room        string    `db:"room"`
	Temperature float32   `db:"temperature"`
	Humidity    float32   `db:"humidity"`
}

func calc_mean(data []SensorData) (float32, float32) {
	var sum_temp float32
	var sum_hum float32
	for _, v := range data {
		sum_temp += v.Temperature
		sum_hum += v.Humidity
	}
	return sum_temp / float32(len(data)), sum_hum / float32(len(data))
}

func GetMeanData(db *sqlx.DB) gin.HandlerFunc {
	return func(context *gin.Context) {
		sensor_data := []SensorData{}
		err := db.Select(&sensor_data, "SELECT * FROM env_data")
		if err != nil {
			log.Fatalln(err)
		}
		mean_temp, mean_hum := calc_mean(sensor_data)
		context.JSON(200, gin.H{
			"mean_temperature": mean_temp,
			"mean_humidity":    mean_hum,
			"len_data":         len(sensor_data),
		})
	}
}

func GetSensorData(db *sqlx.DB) gin.HandlerFunc {
	return func(context *gin.Context) {
		sensor_data := []SensorData{}
		err := db.Select(&sensor_data, "SELECT * FROM env_data ORDER BY ts ASC")
		if err != nil {
			log.Fatalln(err)
		}
		context.JSON(200, sensor_data)
	}
}

func GetLatestSensorData(db *sqlx.DB) gin.HandlerFunc {
	return func(context *gin.Context) {
		sensor_data := []SensorData{}
		err := db.Select(&sensor_data, "SELECT * FROM env_data ORDER BY ts DESC limit 1")
		if err != nil {
			log.Fatalln(err)
		}
		context.JSON(200, sensor_data)
	}
}

func Ping(context *gin.Context) {
	context.JSON(200, gin.H{
		"message": "pong",
	})
}
