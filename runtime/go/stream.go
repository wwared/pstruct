package stream

import (
	"bytes"
	"encoding/binary"
	"io"
	"math"
)

type Struct interface {
	Encode() ([]byte, error)
	EncodeStream(stream *Stream) error
	Decode(data []byte) error
	DecodeStream(stream *Stream) error
}

func read(r io.Reader, size int) ([]byte, error) {
	b := make([]byte, size)
	if _, err := io.ReadFull(r, b); err != nil {
		return nil, err
	}
	return b, nil
}

type Stream struct {
	Reader io.Reader
	Writer io.Writer
}

func (s *Stream) WriteU8(i uint8) error {
	_, err := s.Writer.Write([]byte{i})
	return err
}

func (s *Stream) WriteU16(i uint16, b binary.ByteOrder) error {
	data := make([]byte, 2)
	b.PutUint16(data, i)
	_, err := s.Writer.Write(data)
	return err
}

func (s *Stream) WriteU32(i uint32, b binary.ByteOrder) error {
	data := make([]byte, 4)
	b.PutUint32(data, i)
	_, err := s.Writer.Write(data)
	return err
}

func (s *Stream) WriteU64(i uint64, b binary.ByteOrder) error {
	data := make([]byte, 8)
	b.PutUint64(data, i)
	_, err := s.Writer.Write(data)
	return err
}

func (s *Stream) WriteI8(i int8) error {
	_, err := s.Writer.Write([]byte{byte(i)})
	return err
}

func (s *Stream) WriteI16(i int16, b binary.ByteOrder) error {
	return s.WriteU16(uint16(i), b)
}

func (s *Stream) WriteI32(i int32, b binary.ByteOrder) error {
	return s.WriteU32(uint32(i), b)
}

func (s *Stream) WriteI64(i int64, b binary.ByteOrder) error {
	return s.WriteU64(uint64(i), b)
}

func (s *Stream) WriteF32(f float32, b binary.ByteOrder) error {
	return s.WriteU32(math.Float32bits(f), b)
}

func (s *Stream) WriteF64(f float64, b binary.ByteOrder) error {
	return s.WriteU64(math.Float64bits(f), b)
}

func (s *Stream) WriteBytes(buf []byte) error {
	_, err := s.Writer.Write(buf)
	return err
}

func (s *Stream) WriteString(str string, b binary.ByteOrder) error {
	err := s.WriteU16(uint16(len(str)), b)
	if err != nil {
		return err
	}
	_, err = s.Writer.Write([]byte(str))
	return err
}

func (s *Stream) WriteCString(str string, i int) error {
	b := make([]byte, i)
	copy(b, str)
	_, err := s.Writer.Write(b)
	return err
}

func (s *Stream) WriteCStringUnsized(str string) error {
	_, err := s.Writer.Write([]byte(str))
	if err != nil {
		return err
	}
	err = s.WriteU8(0)
	return err
}

func (s *Stream) ReadU8() (uint8, error) {
	buf, err := read(s.Reader, 1)
	if err != nil {
		return 0, err
	}
	return buf[0], nil
}

func (s *Stream) ReadU16(b binary.ByteOrder) (uint16, error) {
	buf, err := read(s.Reader, 2)
	if err != nil {
		return 0, err
	}
	return b.Uint16(buf), nil
}

func (s *Stream) ReadU32(b binary.ByteOrder) (uint32, error) {
	buf, err := read(s.Reader, 4)
	if err != nil {
		return 0, err
	}
	return b.Uint32(buf), nil
}

func (s *Stream) ReadU64(b binary.ByteOrder) (uint64, error) {
	buf, err := read(s.Reader, 8)
	if err != nil {
		return 0, err
	}
	return b.Uint64(buf), nil
}

func (s *Stream) ReadI8() (int8, error) {
	buf, err := read(s.Reader, 1)
	if err != nil {
		return 0, err
	}
	return int8(buf[0]), nil
}

func (s *Stream) ReadI16(b binary.ByteOrder) (int16, error) {
	buf, err := read(s.Reader, 2)
	if err != nil {
		return 0, err
	}
	return int16(b.Uint16(buf)), nil
}

func (s *Stream) ReadI32(b binary.ByteOrder) (int32, error) {
	buf, err := read(s.Reader, 4)
	if err != nil {
		return 0, err
	}
	return int32(b.Uint32(buf)), nil
}

func (s *Stream) ReadI64(b binary.ByteOrder) (int64, error) {
	buf, err := read(s.Reader, 8)
	if err != nil {
		return 0, err
	}
	return int64(b.Uint64(buf)), nil
}

func (s *Stream) ReadF32(b binary.ByteOrder) (float32, error) {
	f, err := s.ReadU32(b)
	if err != nil {
		return 0, err
	}
	return math.Float32frombits(f), nil
}

func (s *Stream) ReadF64(b binary.ByteOrder) (float64, error) {
	f, err := s.ReadU64(b)
	if err != nil {
		return 0, err
	}
	return math.Float64frombits(f), nil
}

func (s *Stream) ReadBytes(i int) ([]byte, error) {
	return read(s.Reader, int(i))
}

func (s *Stream) ReadString(b binary.ByteOrder) (string, error) {
	i, err := s.ReadU16(b)
	if err != nil {
		return "", err
	}

	buf, err := read(s.Reader, int(i))
	if err != nil {
		return "", err
	}

	return string(buf), nil
}

func (s *Stream) ReadCString(i int) (string, error) {
	buf, err := read(s.Reader, int(i))
	if err != nil {
		return "", err
	}
	end := bytes.IndexByte(buf, '\x00')
	if end == -1 {
		return string(buf), nil
	}
	return string(buf[:end]), nil
}

func (s *Stream) ReadCStringUnsized() (string, error) {
	b, err := s.ReadU8()
	if err != nil {
		return "", err
	}
	bytes := []byte{}
	for b != 0 {
		bytes = append(bytes, b)
		b, err = s.ReadU8()
		if err != nil {
			return "", err
		}
	}
	return string(bytes), nil
}

func NewStream() *Stream {
	return &Stream{}
}

func NewStreamWithReader(reader io.Reader) *Stream {
	return &Stream{Reader: reader}
}

func NewStreamWithWriter(writer io.Writer) *Stream {
	return &Stream{Writer: writer}
}

func NewStreamWithReaderWriter(reader io.Reader, writer io.Writer) *Stream {
	return &Stream{Reader: reader, Writer: writer}
}

func NewStreamWithSlice(data []byte) *Stream {
	// This takes ownership of data -- see NewBuffer docs
	buf := bytes.NewBuffer(data)
	return &Stream{Reader: buf, Writer: buf}
}
