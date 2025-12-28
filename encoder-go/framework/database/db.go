package database

import (
	"encoder/domain"
	"log"

	"gorm.io/driver/postgres"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

type Database struct {
	Db            *gorm.DB
	Dsn           string
	DsnTest       string
	DbType        string
	DbTypeTest    string
	Debug         bool
	AutoMigrateDb bool
	Env           string
}

func NewDb() *Database {
	return &Database{}
}

func NewDbTest() *gorm.DB {
	dbInstance := NewDb()
	dbInstance.Env = "test"
	dbInstance.DbTypeTest = "sqlite3"
	dbInstance.DsnTest = ":memory:"
	dbInstance.AutoMigrateDb = true
	dbInstance.Debug = true

	connection, err := dbInstance.Connect()

	if err != nil {
		log.Fatalf("Test db error: %v", err)
	}

	return connection
}

func (d *Database) Connect() (*gorm.DB, error) {
	var err error

	config := &gorm.Config{}
	if d.Debug {
		config.Logger = logger.Default.LogMode(logger.Info)
	}

	if d.Env != "test" {
		d.Db, err = gorm.Open(d.getDialector(d.DbType, d.Dsn), config)
	} else {
		d.Db, err = gorm.Open(d.getDialector(d.DbTypeTest, d.DsnTest), config)
	}

	if err != nil {
		return nil, err
	}

	if d.AutoMigrateDb {
		d.Db.AutoMigrate(&domain.Video{}, &domain.Job{})
	}

	return d.Db, nil
}

func (d *Database) getDialector(dbType string, dsn string) gorm.Dialector {

	switch dbType {
	case "postgres":
		return postgres.Open(dsn)
	case "sqlite3":
		return sqlite.Open(dsn)
	default:
		return sqlite.Open(dsn)
	}
}
