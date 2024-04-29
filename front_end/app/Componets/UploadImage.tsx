import React, { useState } from 'react';
import { InboxOutlined } from '@ant-design/icons';
import type { GetProp, UploadFile, UploadProps } from 'antd';
import { message, Upload } from 'antd';
import axios from 'axios';
import { UploadFileStatus } from 'antd/es/upload/interface';

const { Dragger } = Upload;
type FileType = Parameters<GetProp<UploadProps, 'beforeUpload'>>[0];

// const UploadImage: React.FC<{useremail: string}> = (props) => {
const UploadImage: React.FC<{ setFileName: Function }> = (props) => {
  const { setFileName } = props;
  const [fileList, setFileList] = useState<UploadFile[]>([]);

  let properties: UploadProps = {
    name: 'file',
    multiple: true,
    fileList: fileList,
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
      const { onSuccess, onError, file } = options;
      axios.post(`/${sessionStorage.getItem('useremail')}/upload_pic`, {
        file: file
      }, {
        headers:{
          'Content-Type': "multipart/form-data"
        }
      }).then(res => {
        onSuccess!(file);
        console.log(res);
      })
      .catch(err=>{
        const error = new Error(err);
        onError!(error);
      });
    },
    onChange(info) {
      const { status } = info.file;
      if (status === 'done') {
        if (info.file.response.code === 200) {
          message.success(`${info.file.name} file uploaded successfully.`);
        }
      } else if (status === 'error') {
        message.error(`${info.file.name} file upload failed.`);
      }
      // console.log(info.fileList);
      setFileList([...info.fileList]);
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