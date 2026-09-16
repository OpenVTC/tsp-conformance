// Command tsp-driver-go wraps github.com/affinidi/affinidi-tsp-go in the TSP
// conformance driver protocol (docs/driver-protocol.md): one JSON request per
// line on stdin, one JSON response per line on stdout.
//
// The driver is a thin adapter. Every message byte, digest and verdict comes
// from the library; the driver only translates JSON.
package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"os"
)

func main() {
	if err := serve(os.Stdin, os.Stdout); err != nil {
		fmt.Fprintln(os.Stderr, "tsp-driver-go:", err)
		os.Exit(1)
	}
}

// serve runs the request loop until EOF on in.
func serve(in io.Reader, out io.Writer) error {
	r := bufio.NewReaderSize(in, 1<<20)
	w := bufio.NewWriterSize(out, 1<<20)
	enc := json.NewEncoder(w)
	enc.SetEscapeHTML(false)
	d := newDriver()
	for {
		line, err := r.ReadBytes('\n')
		if len(line) > 0 {
			if resp := d.handleLine(line); resp != nil {
				if werr := enc.Encode(resp); werr != nil {
					return werr
				}
				if werr := w.Flush(); werr != nil {
					return werr
				}
			}
		}
		if err == io.EOF {
			return nil
		}
		if err != nil {
			return err
		}
	}
}
