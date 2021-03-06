import React from 'react';
import './Loader.css';
import Header from "./Header.js";

function LoadView(props) {
  return(
      <div className = "w-screen h-screen" >
        <Header />
        <div className="flex flex-col text-center h-full w-full">
          <div className="flex-grow"></div>
          <div className="flex flex-row text-center w-full">
            <div className="flex-grow"></div>
            <div className="container-main">
              <Loader />
            </div>
            <div className="flex-grow"></div>
          </div>
          <div className="flex-grow"></div>
        </div>
      </div>
    )
}

function Loader () {
  return (
    <div className="h-full">
      <div className="container left">
        <div className="stick"></div>
        <div className="ball"></div>
      </div>
      <div className="container">
        <div className="stick"></div>
        <div className="ball"></div>
      </div>
      <div className="container">
        <div className="stick"></div>
        <div className="ball"></div>
      </div>
      <div className="container">
        <div className="stick"></div>
        <div className="ball"></div>
      </div>
      <div className="container right">
        <div className="stick"></div>
        <div className="ball"></div>
      </div>
      <p className="text-xl text-center mt-4 text-gray-500">-Waiting-</p>
    </div>
  )
}

export default LoadView