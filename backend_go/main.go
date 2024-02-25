package main

import (
	"log"

	"github.com/gin-gonic/gin"
	"github.com/jmoiron/sqlx"
	_ "github.com/lib/pq"
)

func main() {
	db, err := sqlx.Connect("postgres", "user=postgres dbname=postgres sslmode=disable host=server password=admin")
	if err != nil {
		log.Fatalln(err)
	}

	r := gin.Default()
	r.GET("/sensor_data", GetSensorData(db))
	r.GET("/sensor_data/latest", GetLatestSensorData(db))
	r.GET("/sensor_data/mean", GetMeanData(db))
	r.GET("/ping", Ping)
	r.Run()
}
