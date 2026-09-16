module github.com/affinidi/tsp-conformance/drivers/go

go 1.27

require github.com/affinidi/affinidi-tsp-go v0.0.0

require (
	github.com/cloudflare/circl v1.6.5 // indirect
	golang.org/x/crypto v0.57.0 // indirect
	golang.org/x/sys v0.48.0 // indirect
)

replace github.com/affinidi/affinidi-tsp-go => ../../../affinidi-tsp-go
