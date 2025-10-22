import { Link } from "react-router-dom"
import snapshort4 from '../assets/snapshort4.png'



function OpenFileGuide(){
    return(
        <div className="flex flex-col justify-center gap-5 p-5 h-full bg-[#3D3C3C] font-sans text-white">
            <div className="felx flex-row justify-left text-left text-4xl ">Open File:</div>
            <div className="text-left backdrop-blur-sm">
                A fast file access feature that lets users locate and open files instantly. By typing filenames or keywords, it quickly filters results, showing file paths and details, streamlining workflow and saving time when managing documents, media, or system files.                </div>
            <div className="m-5 px-10 pt-10 rounded-t-xl bg-[#929292]">
                <img src={snapshort4} alt='snapshot1'
                className="flex justify-center items-center rounded-t-xl "/>
            </div>
            <Link to='/GuideEnd'><button className="relative bottom-3 left-170 flex flex-row justify-center py-1 px-3 bg-black rounded-md text-center text-white ">next</button></Link>
        </div>
    )
} 
export default OpenFileGuide 