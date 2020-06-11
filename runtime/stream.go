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
	byteOrder binary.ByteOrder
}

func (s *Stream) WriteU8(i uint8) {
	s.data = append(s.data, i)
}

func (s *Stream) WriteU16(i uint16) {
	s.data = append(s.data, byte(i), byte(i>>8))
}

func (s *Stream) WriteU32(i uint32) {
	s.data = append(s.data, byte(i), byte(i>>8), byte(i>>16), byte(i>>24))
}

func (s *Stream) WriteU64(i uint64) {
	s.data = append(s.data, byte(i), byte(i>>8), byte(i>>16), byte(i>>24), byte(i>>32), byte(i>>40), byte(i>>48), byte(i>>56))
}

func (s *Stream) WriteI8(i int8) {
	s.data = append(s.data, byte(i))
}

func (s *Stream) WriteI16(i int16) {
	s.data = append(s.data, byte(i), byte(i>>8))
}

func (s *Stream) WriteI32(i int32) {
	s.data = append(s.data, byte(i), byte(i>>8), byte(i>>16), byte(i>>24))
}

func (s *Stream) WriteI64(i int64) {
	s.data = append(s.data, byte(i), byte(i>>8), byte(i>>16), byte(i>>24), byte(i>>32), byte(i>>40), byte(i>>48), byte(i>>56))
}

func (s *Stream) WriteF32(f float32) {
	s.WriteU32(math.Float32bits(f))
}

func (s *Stream) WriteF64(f float64) {
	s.WriteU64(math.Float64bits(f))
}

func (s *Stream) WriteString(str string) {
	s.WriteU16(uint16(len(str)))
	s.data = append(s.data, []byte(str)...)
}

func (s *Stream) WriteString64(str string) {
	s.WriteU64(uint64(len(str)))
	s.data = append(s.data, []byte(str)...)
}

func (s *Stream) WriteCString(str string, i uint64) {
	b := make([]byte, i)
	copy(b, str)
	s.WriteBytes(b)
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

func (s *Stream) ReadU16() (uint16, error) {
	pos := s.pos + 2
	defer func() { s.pos = pos }()
	if pos > s.size {
		return 0, fmt.Errorf("unable to read u16 at position %d: out of bounds", s.pos)
	}

	return s.byteOrder.Uint16(s.data[s.pos:pos]), nil
}

func (s *Stream) ReadU32() (uint32, error) {
	pos := s.pos + 4
	defer func() { s.pos = pos }()
	if pos > s.size {
		return 0, fmt.Errorf("unable to read u32 at position %d: out of bounds", s.pos)
	}

	return s.byteOrder.Uint32(s.data[s.pos:pos]), nil
}

func (s *Stream) ReadU64() (uint64, error) {
	pos := s.pos + 8
	defer func() { s.pos = pos }()
	if pos > s.size {
		return 0, fmt.Errorf("unable to read u64 at position %d: out of bounds", s.pos)
	}

	return s.byteOrder.Uint64(s.data[s.pos:pos]), nil
}

func (s *Stream) ReadI8() (int8, error) {
	pos := s.pos + 1
	defer func() { s.pos = pos }()
	if s.pos > s.size {
		return 0, fmt.Errorf("unable to read i8 at position %d: out of bounds", s.pos)
	}

	return int8(s.data[s.pos:pos][0]), nil
}

func (s *Stream) ReadI16() (int16, error) {
	pos := s.pos + 2
	defer func() { s.pos = pos }()
	if pos > s.size {
		return 0, fmt.Errorf("unable to read i16 at position %d: out of bounds", s.pos)
	}

	return int16(s.byteOrder.Uint16(s.data[s.pos:pos])), nil
}

func (s *Stream) ReadI32() (int32, error) {
	pos := s.pos + 4
	defer func() { s.pos = pos }()
	if pos > s.size {
		return 0, fmt.Errorf("unable to read i32 at position %d: out of bounds", s.pos)
	}

	return int32(s.byteOrder.Uint32(s.data[s.pos:pos])), nil
}

func (s *Stream) ReadI64() (int64, error) {
	pos := s.pos + 8
	defer func() { s.pos = pos }()
	if pos > s.size {
		return 0, fmt.Errorf("unable to read i64 at position %d: out of bounds", s.pos)
	}

	return int64(s.byteOrder.Uint64(s.data[s.pos:pos])), nil
}

func (s *Stream) ReadF32() (float32, error) {
	f, err := s.ReadU32()
	if err != nil {
		return 0, err
	}

	return math.Float32frombits(f), nil
}

func (s *Stream) ReadF64() (float64, error) {
	f, err := s.ReadU64()
	if err != nil {
		return 0, err
	}

	return math.Float64frombits(f), nil
}

func (s *Stream) ReadString() (string, error) {
	i, err := s.ReadU16()
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

func (s *Stream) ReadString64() (string, error) {
	i, err := s.ReadU64()
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

	return string(bytes.Trim(b, "\x00")), nil
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
	return &Stream{byteOrder: binary.LittleEndian}
}

func NewStreamWithSize(size uint64) *Stream {
	return &Stream{byteOrder: binary.LittleEndian, data: make([]byte, 0, size)}
}

func NewStreamWithSlice(b []byte) *Stream {
	return &Stream{byteOrder: binary.LittleEndian, data: b}
}

func NewStreamReader(data []byte) *Stream {
	return &Stream{data: data, size: uint64(len(data)), byteOrder: binary.LittleEndian}
}
