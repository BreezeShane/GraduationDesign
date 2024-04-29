import React, { useState } from 'react';
import { InboxOutlined } from '@ant-design/icons';
import type { GetProp, UploadFile, UploadProps } from 'antd';
import { message, Upload } from 'antd';
import axios from 'axios';

const { Dragger } = Upload;
type FileType = Parameters<GetProp<UploadProps, 'beforeUpload'>>[0];

// const UploadImage: React.FC<{useremail: string}> = (props) => {
const UploadImage: React.FC<{ setFileName: Function }> = (props) => {
  const { setFileName } = props;

  let properties: UploadProps = {
    name: 'file',
    multiple: true,
    beforeUpload(file, FileList) {
      if (sessionStorage.getItem('useremail')) {
        setFileName(file);
        return true;
      } else {
        message.error("You should sign in first!");
        setFileName("");
        return false;
      }
    },
    customRequest(options) {
      const file_obj = options.file;
      axios.post(`/${sessionStorage.getItem('useremail')}/upload_pic`, {
        file: file_obj
      }, {
        headers:{
          'Content-Type': "multipart/form-data"
        }
      }).then(function (response) {
        console.log(response);
      });

    },
    onChange(info) {
      const { status } = info.file;
      if (status !== 'uploading') {
        console.log(info.file, info.fileList);
      }
      if (status === 'done') {
        message.success(`${info.file.name} file uploaded successfully.`);
      } else if (status === 'error') {
        message.error(`${info.file.name} file upload failed.`);
      }
    },
    onDrop(e) {
      console.log('Dropped files', e.dataTransfer.files);
    },
  };

  return (
    <>
      <Dragger {...properties}>
          <p className="ant-upload-drag-icon">
              <InboxOutlined />
          </p>
          <p className="ant-upload-text">Click or drag file to this area to upload</p>
          <p className="ant-upload-hint">
              Support for a single or bulk upload. Strictly prohibited from uploading company data or other
              banned files.
          </p>
      </Dragger>
    </>
  );
};

export default UploadImage;