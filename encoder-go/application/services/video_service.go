package services

import (
	"context"
	"encoder/application/repositories"
	"encoder/domain"
	"fmt"
	"io"
	"log"
	"os"
	"os/exec"
	"time"

	"cloud.google.com/go/storage"
)

const cmdTimeout = 10 * time.Minute

type VideoService struct {
	Video           *domain.Video
	VideoRepository repositories.VideoRepository
}

func NewVideoService() VideoService {
	return VideoService{}
}

func (v *VideoService) getLocalStoragePath() string {
	path := os.Getenv(EnvLocalStoragePath)
	if path == "" {
		return "/tmp"
	}
	return path
}

func (v *VideoService) Download(bucketName string) error {
	ctx := context.Background()

	client, err := storage.NewClient(ctx)
	if err != nil {
		return err
	}
	defer client.Close()

	bkt := client.Bucket(bucketName)
	obj := bkt.Object(v.Video.FilePath)

	r, err := obj.NewReader(ctx)
	if err != nil {
		return err
	}
	defer r.Close()

	filePath := v.getLocalStoragePath() + "/" + v.Video.ID + ".mp4"
	f, err := os.Create(filePath)
	if err != nil {
		return err
	}
	defer f.Close()

	_, err = io.Copy(f, r)
	if err != nil {
		return err
	}

	log.Printf("video %v has been stored", v.Video.ID)

	return nil
}

func (v *VideoService) Fragment() error {
	localPath := v.getLocalStoragePath()

	err := os.MkdirAll(localPath+"/"+v.Video.ID, 0750)
	if err != nil {
		return err
	}

	source := localPath + "/" + v.Video.ID + ".mp4"
	target := localPath + "/" + v.Video.ID + ".frag"

	ctx, cancel := context.WithTimeout(context.Background(), cmdTimeout)
	defer cancel()

	cmd := exec.CommandContext(ctx, "mp4fragment", source, target)

	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("mp4fragment failed: %w", err)
	}

	printOutPut(output)

	return nil
}

func (v *VideoService) Encode() error {
	localPath := v.getLocalStoragePath()

	cmdArgs := []string{
		localPath + "/" + v.Video.ID + ".frag",
		"--use-segment-timeline",
		"-o",
		localPath + "/" + v.Video.ID,
		"-f",
		"--exec-dir",
		"/opt/bento4/bin/",
	}

	ctx, cancel := context.WithTimeout(context.Background(), cmdTimeout)
	defer cancel()

	cmd := exec.CommandContext(ctx, "mp4dash", cmdArgs...)

	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("mp4dash failed: %w", err)
	}

	printOutPut(output)

	return nil
}

func (v *VideoService) Finish() error {
	localPath := v.getLocalStoragePath()

	err := os.Remove(localPath + "/" + v.Video.ID + ".mp4")
	if err != nil {
		log.Println("error removing mp4 ", v.Video.ID, ".mp4")
		return err
	}

	err = os.Remove(localPath + "/" + v.Video.ID + ".frag")
	if err != nil {
		log.Println("error removing frag ", v.Video.ID, ".frag")
		return err
	}

	err = os.RemoveAll(localPath + "/" + v.Video.ID)
	if err != nil {
		log.Println("error removing folder ", v.Video.ID)
		return err
	}

	log.Println("Files have been removed:", v.Video.ID)

	return nil
}

func (v *VideoService) InsertVideo() error {
	_, err := v.VideoRepository.Insert(v.Video)

	if err != nil {
		return err
	}

	return nil
}

func printOutPut(out []byte) {
	if len(out) > 0 {
		log.Printf("=====> Output: %s", string(out))
	}
}
