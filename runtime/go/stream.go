package stream

import (
	"bytes"
	"encoding/binary"
	"encoding/hex"
	"fmt"
	"math"
)

// TODO: big endian write support
// TODO: streaming

type Struct interface {
	Encode() []byte
	EncodeStream(stream *Stream)
	Decode(data []byte) error
	DecodeStream(stream *Stream) error
}

type Stream struct {
	data      []byte
	size      uint64
	pos       uint64
}

func (s *Stream) WriteU8(i uint8) {
	s.data = append(s.data, i)
}

func (s *Stream) WriteU16(i uint16, b binary.ByteOrder) {
	pos := len(s.data)
	s.data = append(s.data, 0, 0)
	b.PutUint16(s.data[pos:], i)
}

func (s *Stream) WriteU32(i uint32, b binary.ByteOrder) {
	pos := len(s.data)
	s.data = append(s.data, 0, 0, 0, 0)
	b.PutUint32(s.data[pos:], i)
}

func (s *Stream) WriteU64(i uint64, b binary.ByteOrder) {
	pos := len(s.data)
	s.data = append(s.data, 0, 0, 0, 0, 0, 0, 0, 0)
	b.PutUint64(s.data[pos:], i)
}

func (s *Stream) WriteI8(i int8) {
	s.data = append(s.data, byte(i))
}

func (s *Stream) WriteI16(i int16, b binary.ByteOrder) {
	s.WriteU16(uint16(i), b)
}

func (s *Stream) WriteI32(i int32, b binary.ByteOrder) {
	s.WriteU32(uint32(i), b)
}

func (s *Stream) WriteI64(i int64, b binary.ByteOrder) {
	s.WriteU64(uint64(i), b)
}

func (s *Stream) WriteF32(f float32, b binary.ByteOrder) {
	s.WriteU32(math.Float32bits(f), b)
}

func (s *Stream) WriteF64(f float64, b binary.ByteOrder) {
	s.WriteU64(math.Float64bits(f), b)
}

func (s *Stream) WriteString(str string, b binary.ByteOrder) {
	s.WriteU16(uint16(len(str)), b)
	s.data = append(s.data, []byte(str)...)
}

func (s *Stream) WriteString64(str string, b binary.ByteOrder) {
	s.WriteU64(uint64(len(str)), b)
	s.data = append(s.data, []byte(str)...)
}

func (s *Stream) WriteCString(str string, i uint64) {
	b := make([]byte, i)
	copy(b, str)
	s.WriteBytes(b)
}

func (s *Stream) WriteCStringUnsized(str string) {
	s.WriteBytes([]byte(str))
	s.data = append(s.data, 0)
}

func (s *Stream) WriteBytes(b []byte) {
	s.data = append(s.data, b...)
}

func (s *Stream) WriteHex(h string) {
	d, err := hex.DecodeString(h)
	if err != nil {
		panic(err)
	}

	s.WriteBytes(d)
}

func (s *Stream) ReadU8() (uint8, error) {
	pos := s.pos + 1
	defer func() { s.pos = pos }()
	if s.pos > s.size {
		return 0, fmt.Errorf("unable to read u8 at position %d: out of bounds", s.pos)
	}

	return s.data[s.pos:pos][0], nil
}

func (s *Stream) ReadU16(b binary.ByteOrder) (uint16, error) {
	pos := s.pos + 2
	defer func() { s.pos = pos }()
	if pos > s.size {
		return 0, fmt.Errorf("unable to read u16 at position %d: out of bounds", s.pos)
	}

	return b.Uint16(s.data[s.pos:pos]), nil
}

func (s *Stream) ReadU32(b binary.ByteOrder) (uint32, error) {
	pos := s.pos + 4
	defer func() { s.pos = pos }()
	if pos > s.size {
		return 0, fmt.Errorf("unable to read u32 at position %d: out of bounds", s.pos)
	}

	return b.Uint32(s.data[s.pos:pos]), nil
}

func (s *Stream) ReadU64(b binary.ByteOrder) (uint64, error) {
	pos := s.pos + 8
	defer func() { s.pos = pos }()
	if pos > s.size {
		return 0, fmt.Errorf("unable to read u64 at position %d: out of bounds", s.pos)
	}

	return b.Uint64(s.data[s.pos:pos]), nil
}

func (s *Stream) ReadI8() (int8, error) {
	pos := s.pos + 1
	defer func() { s.pos = pos }()
	if s.pos > s.size {
		return 0, fmt.Errorf("unable to read i8 at position %d: out of bounds", s.pos)
	}

	return int8(s.data[s.pos:pos][0]), nil
}

func (s *Stream) ReadI16(b binary.ByteOrder) (int16, error) {
	pos := s.pos + 2
	defer func() { s.pos = pos }()
	if pos > s.size {
		return 0, fmt.Errorf("unable to read i16 at position %d: out of bounds", s.pos)
	}

	return int16(b.Uint16(s.data[s.pos:pos])), nil
}

func (s *Stream) ReadI32(b binary.ByteOrder) (int32, error) {
	pos := s.pos + 4
	defer func() { s.pos = pos }()
	if pos > s.size {
		return 0, fmt.Errorf("unable to read i32 at position %d: out of bounds", s.pos)
	}

	return int32(b.Uint32(s.data[s.pos:pos])), nil
}

func (s *Stream) ReadI64(b binary.ByteOrder) (int64, error) {
	pos := s.pos + 8
	defer func() { s.pos = pos }()
	if pos > s.size {
		return 0, fmt.Errorf("unable to read i64 at position %d: out of bounds", s.pos)
	}

	return int64(b.Uint64(s.data[s.pos:pos])), nil
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

func (s *Stream) ReadString(b binary.ByteOrder) (string, error) {
	i, err := s.ReadU16(b)
	if err != nil {
		return "", fmt.Errorf("unable to read string at position %d: out of bounds", s.pos)
	}

	i2 := uint64(i)
	pos := s.pos + i2
	defer func() { s.pos = pos }()

	if pos > s.size {
		return "", fmt.Errorf("unable to read string at position %d: out of bounds", s.pos)
	}

	return string(s.data[s.pos:pos]), nil
}

func (s *Stream) ReadString64(b binary.ByteOrder) (string, error) {
	i, err := s.ReadU64(b)
	if err != nil {
		return "", fmt.Errorf("unable to read string64 at position %d: out of bounds", s.pos)
	}

	pos := s.pos + i
	defer func() { s.pos = pos }()

	if pos > s.size {
		return "", fmt.Errorf("unable to read string64 at position %d: out of bounds", s.pos)
	}

	return string(s.data[s.pos:pos]), nil
}

func (s *Stream) ReadCString(i uint64) (string, error) {
	b, err := s.ReadBytes(i)
	if err != nil {
		return "", fmt.Errorf("unable to read C string (%d bytes) at position %d: out of bounds", i, s.pos)
	}
	end := bytes.IndexByte(b, '\x00')
	if end == -1 {
		return string(b), nil
	}
	return string(b[:end]), nil
}

func (s *Stream) ReadCStringUnsized() (string, error) {
	b, err := s.ReadU8()
	if err != nil {
		return "", fmt.Errorf("unable to read C unsized string at position %d: out of bounds", s.pos)
	}
	bytes := []byte{}
	for b != 0 {
		bytes = append(bytes, b)
		b, err = s.ReadU8()
		if err != nil {
			return "", fmt.Errorf("unable to read C string at position %d: out of bounds", s.pos)
		}
	}
	return string(bytes), nil
}

func (s *Stream) ReadBytes(i uint64) ([]byte, error) {
	pos := s.pos + i
	defer func() { s.pos = pos }()

	if pos > s.size {
		return nil, fmt.Errorf("unable to read %d bytes at position %d: out of bounds", i, s.pos)
	}

	return s.data[s.pos:pos], nil
}

func (s *Stream) GetData() []byte {
	return s.data
}

func NewStream() *Stream {
	return &Stream{}
}

func NewStreamWithSize(size uint64) *Stream {
	return &Stream{data: make([]byte, 0, size)}
}

func NewStreamWithSlice(b []byte) *Stream {
	return &Stream{data: b}
}

func NewStreamReader(data []byte) *Stream {
	return &Stream{data: data, size: uint64(len(data))}
}
