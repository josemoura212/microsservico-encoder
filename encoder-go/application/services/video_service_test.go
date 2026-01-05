package services_test

import (
	"encoder/application/repositories"
	"encoder/application/services"
	"encoder/domain"
	"encoder/framework/database"
	"os"
	"testing"
	"time"

	"github.com/joho/godotenv"
	uuid "github.com/satori/go.uuid"
	"github.com/stretchr/testify/require"
)

func prepare() (*domain.Video, repositories.VideoRepositoryDb) {

	db := database.NewDbTest()

	video := domain.NewVideo()
	video.ID = uuid.NewV4().String()
	video.FilePath = "videos/3fa3291e-5daf-4386-9a67-69d19e1690c5/videos/3fa3291e-5daf-4386-9a67-69d19e1690c5-b8c187dd77c950e9b117bcc19e35a9005e45001593f7f4260040cee47d77faa0.mp4"
	video.ResourceID = "3fa3291e-5daf-4386-9a67-69d19e1690c5"
	video.CreatedAt = time.Now()

	repo := repositories.VideoRepositoryDb{Db: db}

	return video, repo
}

func init() {
	os.MkdirAll("../../tmp", os.ModePerm)

	err := godotenv.Overload("../../.env")
	if err != nil {
		panic("Error loading .env file")
	}
}

func TestVideoServiceDownload(t *testing.T) {
	video, repo := prepare()

	videoService := services.NewVideoService()

	videoService.Video = video
	videoService.VideoRepository = &repo

	err := videoService.Download("micro-admin-typescript-josemoura212")

	require.Nil(t, err)

	err = videoService.Fragment()
	require.Nil(t, err)
}
