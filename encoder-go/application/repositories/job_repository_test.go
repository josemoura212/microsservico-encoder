package repositories_test

import (
	"encoder/application/repositories"
	"encoder/domain"
	"encoder/framework/database"
	"testing"
	"time"

	uuid "github.com/satori/go.uuid"
	"github.com/stretchr/testify/require"
)

func TestJobRepositoryInsert(t *testing.T) {
	db := database.NewDbTest()

	video := domain.NewVideo()
	video.ID = uuid.NewV4().String()
	video.FilePath = "/path/to/video.mp4"
	video.ResourceID = "video"
	video.CreatedAt = time.Now()

	repo := repositories.NewVideoRepositoryDb(db)
	repo.Insert(video)

	job, err := domain.NewJob("output_path", "Pending", video)

	require.Nil(t, err)

	repoJob := repositories.NewJobRepositoryDb(db)

	repoJob.Insert(job)

	j, err := repoJob.Find(job.ID)

	require.NotEmpty(t, j.ID)
	require.Nil(t, err)
	require.Equal(t, job.ID, j.ID)
	require.Equal(t, j.Video.ID, video.ID)
}

func TestJobRepositoryUpdate(t *testing.T) {
	db := database.NewDbTest()

	video := domain.NewVideo()
	video.ID = uuid.NewV4().String()
	video.FilePath = "/path/to/video.mp4"
	video.ResourceID = "video"
	video.CreatedAt = time.Now()

	repo := repositories.NewVideoRepositoryDb(db)
	repo.Insert(video)

	job, err := domain.NewJob("output_path", "Pending", video)

	require.Nil(t, err)

	repoJob := repositories.NewJobRepositoryDb(db)

	repoJob.Insert(job)

	job.Status = "Completed"

	repoJob.Update(job)

	j, err := repoJob.Find(job.ID)

	require.NotEmpty(t, j.ID)
	require.Nil(t, err)
	require.Equal(t, job.ID, j.ID)
	require.Equal(t, j.Video.ID, video.ID)
	require.Equal(t, "Completed", j.Status)
}
