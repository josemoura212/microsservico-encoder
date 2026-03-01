package services_test

import (
	"encoder/application/services"
	"os"
	"testing"

	"github.com/joho/godotenv"
	"github.com/stretchr/testify/require"
)

func init() {
	os.MkdirAll("../../tmp", os.ModePerm)

	err := godotenv.Overload("../../.env")
	if err != nil {
		panic("Error loading .env file")
	}
}

func TestVideoServiceUpload(t *testing.T) {
	video, repo := prepare()

	videoService := services.NewVideoService()

	videoService.Video = video
	videoService.VideoRepository = &repo

	err := videoService.Download(services.BucketName)

	require.Nil(t, err)

	err = videoService.Fragment()
	require.Nil(t, err)

	err = videoService.Encode()
	require.Nil(t, err)

	videoUpload := services.NewVideoUpload()
	videoUpload.OutputBucket = services.BucketName
	videoUpload.VideoPath = os.Getenv(services.EnvLocalStoragePath) + "/" + video.ID

	doneUpload := make(chan string)

	go func() {
		err := videoUpload.ProcessUpload(50, doneUpload)
		if err != nil {
			doneUpload <- err.Error()
		}
	}()

	result := <-doneUpload

	require.Equal(t, "upload completed", result)

	err = videoService.Finish()
	require.Nil(t, err)
}
