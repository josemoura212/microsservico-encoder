package services

import (
	"context"
	"io"
	"log"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"cloud.google.com/go/storage"
)

type VideoUpload struct {
	Paths        []string
	VideoPath    string
	OutputBucket string
	Erros        []string
}

func NewVideoUpload() *VideoUpload {
	return &VideoUpload{}
}

func (vu *VideoUpload) UploadObject(objectPath string, client *storage.Client, ctx context.Context) error {
	path := strings.Split(objectPath, os.Getenv(EnvLocalStoragePath)+"/")

	f, err := os.Open(objectPath)

	if err != nil {
		return err
	}

	defer f.Close()

	wc := client.Bucket(vu.OutputBucket).Object(path[1]).NewWriter(ctx)
	wc.ACL = []storage.ACLRule{{Entity: storage.AllUsers, Role: storage.RoleReader}}

	if _, err = io.Copy(wc, f); err != nil {
		return err
	}

	if err := wc.Close(); err != nil {
		return err
	}

	return nil
}

func (vu *VideoUpload) loadPaths() error {
	err := filepath.Walk(vu.VideoPath, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		if !info.IsDir() {
			vu.Paths = append(vu.Paths, path)
		}

		return nil
	})

	if err != nil {
		return err
	}

	return nil
}

func (vu *VideoUpload) ProcessUpload(concurrency int, doneUpload chan string) error {
	err := vu.loadPaths()

	if err != nil {
		return err
	}

	in := make(chan int, runtime.NumCPU())
	returnChannel := make(chan string, len(vu.Paths))

	uploadClient, ctx, err := getClientUpload()

	if err != nil {
		return err
	}

	for process := 0; process < concurrency; process++ {
		go vu.uploadWorker(in, returnChannel, uploadClient, ctx)
	}

	go func() {
		for i := 0; i < len(vu.Paths); i++ {
			in <- i
		}
		close(in)
	}()

	for i := 0; i < len(vu.Paths); i++ {
		r := <-returnChannel
		if r != "" {
			doneUpload <- r
			return nil
		}
	}

	doneUpload <- "upload completed"

	return nil
}

func (vu *VideoUpload) uploadWorker(in chan int, returnChannel chan string, uploadClient *storage.Client, ctx context.Context) {

	for x := range in {
		err := vu.UploadObject(vu.Paths[x], uploadClient, ctx)

		if err != nil {
			vu.Erros = append(vu.Erros, vu.Paths[x])
			log.Printf("error during the upload: %v. Error: %v", vu.Paths[x], err)

			returnChannel <- err.Error()
			continue
		}

		returnChannel <- ""
	}
}

func getClientUpload() (*storage.Client, context.Context, error) {

	ctx := context.Background()

	client, err := storage.NewClient(ctx)

	if err != nil {
		return nil, nil, err
	}

	return client, ctx, nil
}
