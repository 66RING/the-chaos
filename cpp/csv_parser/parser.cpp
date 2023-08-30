#include <assert.h>
#include <fstream>
#include <iostream>
#include <memory>
#include <sstream>
#include <vector>

template <typename T> class Tensor {};

template <> class Tensor<float> {
public:
  Tensor(uint32_t rows, uint32_t cols)
      : data_(rows, std::vector<float>(cols)) {}

  std::vector<uint32_t> shape() {
    std::vector<uint32_t> s;
    s.push_back(data_.size());
    s.push_back(data_[0].size());
    return s;
  };

  uint32_t rows() { return data_.size(); }

  uint32_t cols() {
    assert(data_.size() > 0);
    return data_[0].size();
  }

  std::vector<std::vector<float>> &data() { return data_; }

private:
  std::vector<std::vector<float>> data_;
};

class CSVDataLoader {
public:
  static std::shared_ptr<Tensor<float>> LoadData(const std::string &file_path,
                                                 char split_char = ',') {
    // check path
    assert(!file_path.empty());
    // open file stream
    std::ifstream in(file_path);
    assert(in.is_open() && in.good());

    // get max rows and cols
    // NOTE: decouple tips in cpp
    auto [rows, cols] = CSVDataLoader::GetMatrixSize(in, split_char);

    // init target tensor
    std::shared_ptr<Tensor<float>> tensor =
        std::make_shared<Tensor<float>>(rows, cols);
    auto &data = tensor->data();

    // helper value
    std::string line_str;
    std::stringstream line_stream;

    // current row
    size_t row = 0;
    while (in.good()) {
      // read each line
      std::getline(in, line_str);
      if (line_str.empty()) {
        break;
      }

      // read each token of this line
      std::string token;
      // convert string as stream to read
      line_stream.clear();
      line_stream.str(line_str);

      size_t col = 0;
      while (line_stream.good()) {
        std::getline(line_stream, token, split_char);
        try {
          data[row][col] = std::stof(token);
        } catch (std::exception &e) {
          continue;
        }
        col += 1;
        assert(col <= cols);
      }

      row += 1;
      assert(row <= rows);
    }

    return tensor;
  }

  static std::shared_ptr<Tensor<float>>
  LoadDataWithHeader(const std::string &file_path,
                     std::vector<std::string> &headers, char split_char = ',') {
    // check path
    assert(!file_path.empty());
    // open file stream
    std::ifstream in(file_path);
    assert(in.is_open() && in.good());

    // get max rows and cols
    // NOTE: decouple tips in cpp
    auto [rows, cols] = CSVDataLoader::GetMatrixSize(in, split_char);

    // should container header line
    assert(rows >= 1);

    // init target tensor
    std::shared_ptr<Tensor<float>> tensor =
        std::make_shared<Tensor<float>>(rows - 1, cols);
    auto &data = tensor->data();

    // helper value
    std::string line_str;
    std::stringstream line_stream;

    // current row
    size_t row = 0;
    while (in.good()) {
      // read each line
      std::getline(in, line_str);
      if (line_str.empty()) {
        break;
      }

      // read each token of this line
      std::string token;
      // convert string as stream to read
      line_stream.clear();
      line_stream.str(line_str);

      size_t col = 0;
      while (line_stream.good()) {
        std::getline(line_stream, token, split_char);
        try {
          if (row == 0) {
            headers.push_back(token);
          } else {
            data[row - 1][col] = std::stof(token);
          }
        } catch (std::exception &e) {
          continue;
        }
        col += 1;
        assert(col <= cols);
      }

      row += 1;
      assert(row - 1 <= rows);
    }

    return tensor;
  }

private:
  // get matrix size
  static std::pair<size_t, size_t> GetMatrixSize(std::ifstream &file,
                                                 char split_char) {
    // reset error state
    file.clear();

    size_t col_num = 0;
    size_t row_num = 0;

    // store origin state for reverse later.
    auto start_pos = file.tellg();
    std::string line_str;
    std::stringstream line_stream;
    while (file.good()) {
      std::getline(file, line_str);
      if (line_str.empty()) {
        break;
      }

      std::string token;
      size_t line_col = 0;
      line_stream.clear();
      line_stream.str(line_str);

      // read each col
      while (line_stream.good()) {
        std::getline(line_stream, token, split_char);
        line_col += 1;
      }

      if (line_col > col_num) {
        col_num = line_col;
      }

      row_num += 1;
    }

    // restore file state
    file.clear();
    file.seekg(start_pos);
    return {row_num, col_num};
  }
};

int main() {
  // without header test
  {
    const std::string &file_path = "./data1.csv";
    std::shared_ptr<Tensor<float>> tensor =
        CSVDataLoader::LoadData(file_path, ',');
    uint32_t index = 1;
    uint32_t rows = tensor->rows();
    uint32_t cols = tensor->cols();
    assert(rows == 3);
    assert(cols == 6);
    const auto &data = tensor->data();
    for (uint32_t r = 0; r < rows; ++r) {
      for (uint32_t c = 0; c < cols; ++c) {
        assert(data[r][c] == index);
        index += 1;
      }
    }
  }

  // with header test
  {
    const std::string &file_path = "./data2.csv";
    std::vector<std::string> headers;
    std::shared_ptr<Tensor<float>> tensor =
        CSVDataLoader::LoadDataWithHeader(file_path, headers, ',');
	auto data = tensor->data();

    uint32_t index = 1;
    uint32_t rows = tensor->rows();
    uint32_t cols = tensor->cols();
    assert(rows == 3);
    assert(cols == 3);
    assert(headers.size() == 3);

    assert(headers.at(0) == "ROW1");
    assert(headers.at(1) == "ROW2");
    assert(headers.at(2) == "ROW3");

    for (uint32_t r = 0; r < rows; ++r) {
      for (uint32_t c = 0; c < cols; ++c) {
        assert(data[r][c] == index);
        index += 1;
      }
    }
  }
  return 0;
}
